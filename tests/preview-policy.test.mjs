import {test} from 'node:test';
import assert from 'node:assert/strict';
import {allowsTestingPreview} from '../scripts/preview-policy.mjs';

test('testing previews require an explicit and internally consistent engine declaration', () => {
  assert.equal(allowsTestingPreview({status:'preview',inheritedEngineRemoved:true}),true);
  assert.equal(allowsTestingPreview({status:'inherited-engine-retained',inheritedEngineRemoved:false,previewAllowed:true}),true);
  for(const value of [null,{}, {status:'preview',inheritedEngineRemoved:false}, {status:'inherited-engine-retained',inheritedEngineRemoved:true,previewAllowed:true}, {status:'inherited-engine-retained',inheritedEngineRemoved:false}, {status:'ready',inheritedEngineRemoved:true,windowsAcceptance:false}]) {
    assert.equal(allowsTestingPreview(value),false);
  }
});
