// Preview declarations must describe the engine actually being distributed.
export function allowsTestingPreview(migration) {
  return migration?.status === 'preview' && migration.inheritedEngineRemoved === true
    || migration?.status === 'inherited-engine-retained'
      && migration.inheritedEngineRemoved === false
      && migration.previewAllowed === true;
}
