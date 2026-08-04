const tsconfig = require("./tsconfig.eslint.json");

require("ts-node").register(tsconfig);

const config = require("./.eslintrc.ts");

module.exports = config;
