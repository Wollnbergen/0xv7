module.exports = {
  testEnvironment: 'node',
  testMatch: [
    '**/tests/**/*.test.js',
    '**/api/tests/**/*.test.js',
    '**/__tests__/**/*.js'
  ],
  testPathIgnorePatterns: [
    '/node_modules/',
    '/cosmos-data/',
    '/target/',
    '/dist/',
    '/build/'
  ],
  watchPathIgnorePatterns: [
    '/cosmos-data/',
    '/node_modules/',
    '/.git/'
  ],
  modulePathIgnorePatterns: [
    '/cosmos-data/'
  ],
  coveragePathIgnorePatterns: [
    '/node_modules/',
    '/cosmos-data/',
    '/tests/'
  ],
  watchman: false
};
