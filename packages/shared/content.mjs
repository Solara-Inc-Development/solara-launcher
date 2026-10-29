/** Public Modrinth metadata only. Never attach Solara or Minecraft credentials. */
export const contentKinds = ['mod', 'modpack', 'resourcepack', 'shader'];
const identifier = /^[A-Za-z0-9_-]{1,100}$/;
/** @param {unknown} value */
export function providerId(value) {
  if (typeof value !== 'string' || !identifier.test(value)) throw new Error('Invalid content identifier');
  return value;
}
/** @param {unknown} value */
export function contentKind(value) {
  if (typeof value !== 'string' || !contentKinds.includes(value)) throw new Error('Unsupported content type');
  return value;
}
/** @param {unknown} value */
export function providerIcon(value) {
  if (typeof value !== 'string') return null;
  try {
    const url = new URL(value);
    return url.protocol === 'https:' && url.hostname === 'cdn.modrinth.com' && !url.username && !url.password && !url.port ? url.href : null;
  } catch { return null; }
}
/** @param {string} query @param {string} kind */
export function searchUrl(query, kind) {
  const url = new URL('https://api.modrinth.com/v2/search');
  url.search = new URLSearchParams({query: query.trim().slice(0, 200), limit: '18', facets: JSON.stringify([[`project_type:${contentKind(kind)}`]])}).toString();
  return url.href;
}
/** @param {string} project */
export function versionsUrl(project) { return `https://api.modrinth.com/v2/project/${providerId(project)}/version`; }
/** @param {any} hit */
export function projectUrl(hit) { return `https://modrinth.com/${contentKind(hit.project_type)}/${providerId(hit.slug || hit.project_id)}`; }
/** @param {any} data @param {string} kind */
export function normalizeHits(data, kind) {
  contentKind(kind);
  if (!data || !Array.isArray(data.hits)) throw new Error('Content provider returned invalid search data');
  return data.hits.slice(0, 18).filter((/** @type {any} */ hit) => hit && hit.project_type === kind && typeof hit.project_id === 'string' && identifier.test(hit.project_id) && typeof hit.title === 'string').map((/** @type {any} */ hit) => ({
    project_id: hit.project_id, project_type: kind, slug: typeof hit.slug === 'string' && identifier.test(hit.slug) ? hit.slug : hit.project_id,
    title: hit.title.slice(0, 160), description: typeof hit.description === 'string' ? hit.description.slice(0, 1000) : '',
    author: typeof hit.author === 'string' ? hit.author.slice(0, 100) : '', icon_url: providerIcon(hit.icon_url),
    downloads: Number.isSafeInteger(hit.downloads) && hit.downloads >= 0 ? hit.downloads : null,
  }));
}
/** @param {any} data @param {string} project */
export function normalizeVersions(data, project) {
  providerId(project);
  if (!Array.isArray(data)) throw new Error('Content provider returned invalid version data');
  return data.filter(v => v && v.project_id === project && typeof v.id === 'string' && identifier.test(v.id) && typeof v.name === 'string' && ['release', 'beta', 'alpha'].includes(v.version_type) && Array.isArray(v.files) && v.files.some((/** @type {any} */ f) => f && typeof f.filename === 'string' && f.filename.endsWith('.mrpack'))).slice(0, 500).map(v => ({
    id: v.id, name: v.name.slice(0, 160), type: v.version_type,
    games: Array.isArray(v.game_versions) ? v.game_versions.filter((/** @type {any} */ x) => typeof x === 'string').slice(0, 10).join(', ') : '',
    loaders: Array.isArray(v.loaders) ? v.loaders.filter((/** @type {any} */ x) => typeof x === 'string').slice(0, 5).join(', ') : '',
    published: Number.isFinite(Date.parse(v.date_published)) ? Date.parse(v.date_published) : 0,
  })).sort((a, b) => b.published - a.published);
}
/** @param {string} url @param {AbortSignal} signal */
export async function providerJson(url, signal) {
  const response = await fetch(url, {credentials: 'omit', redirect: 'error', signal});
  if (!response.ok) throw new Error(`Content provider returned ${response.status}`);
  const text = await response.text();
  if (text.length > 2_000_000) throw new Error('Content provider response is too large');
  return JSON.parse(text);
}
