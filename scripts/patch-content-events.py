#!/usr/bin/env python3
"""Preserve headless Theseus instance events for Solara's local install activity."""
from pathlib import Path


def patch(lib: Path) -> None:
    root = lib / 'src/lib.rs'
    source = root.read_text()
    marker = 'pub use event::{InstancePayload, InstancePayloadType, emit::emit_instance};'
    if marker not in source:
        anchor = 'pub use logger::start_logger;'
        if source.count(anchor) != 1:
            raise RuntimeError('Pinned event export context changed; review before patching')
        root.write_text(source.replace(anchor, '#[cfg(not(feature = "tauri"))]\n' + marker + '\n' + anchor, 1))
    state = lib / 'src/event/mod.rs' 
    source = state.read_text()
    if 'pub solara_instance_events:' not in source:
        field = '    pub loading_bars: DashMap<Uuid, LoadingBar>,'
        constructor = '                Ok(Arc::new(Self {\n                    loading_bars: DashMap::new(),\n                }))'
        if source.count(field) != 1 or source.count(constructor) != 1:
            raise RuntimeError('Pinned headless EventState context changed; review before patching')
        source = source.replace(field, field + '\n    #[cfg(not(feature = "tauri"))]\n    pub solara_instance_events: tokio::sync::broadcast::Sender<InstancePayload>,', 1)
        source = source.replace(constructor, '                Ok(Arc::new(Self {\n                    loading_bars: DashMap::new(),\n                    solara_instance_events: tokio::sync::broadcast::channel(256).0,\n                }))', 1)
        state.write_text(source)
    emitter = lib / 'src/event/emit.rs'
    source = emitter.read_text()
    start = source.index('pub async fn emit_instance(')
    end = source.index('pub async fn emit_instance_groups_changed(', start)
    block = source[start:end]
    if 'solara_instance_events.send' not in block:
        if block.count('    Ok(())') != 1:
            raise RuntimeError('Pinned instance emitter context changed; review before patching')
        block = block.replace('    Ok(())', '''    #[cfg(not(feature = "tauri"))]
    {
        let _ = crate::EventState::get().solara_instance_events.send(crate::event::InstancePayload {
            instance_id: instance_id.to_string(),
            event,
        });
    }
    Ok(())''', 1)
        emitter.write_text(source[:start] + block + source[end:])


if __name__ == '__main__':
    patch(Path(__file__).resolve().parents[1] / '.upstream/modrinth/packages/app-lib')
    print('Applied headless content-event patch to pinned local engine')
