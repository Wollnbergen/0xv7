const assert = require('assert');

describe('Hello World Test', () => {
    it('should return hello world', () => {
        const result = 'hello world';
        assert.strictEqual(result, 'hello world');
    });
});