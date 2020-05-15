module.exports = () => {
  process.env.VUE_CLI_BABEL_TRANSPILE_MODULES = true;

  return {
    files: ["src/**/*", "jest.config.js", "package.json", "tsconfig.json"],

    tests: ["tests/**/*.spec.ts", "tests/**/*.spec.js"],

    env: {
      type: "node"
    },

    preprocessors: {
      "**/*.js?(x)": file =>
        require("babel-core").transform(file.content, {
          babelrc: true,
          sourceMap: true,
          compact: false,
          filename: file.path,
          plugins: [
            "babel-plugin-jest-hoist",
            "@babel/plugin-syntax-dynamic-import"
          ]
        })
    },

    setup(wallaby) {
      const jestConfig = require("./package").jest || require("./jest.config");
      jestConfig.transform = {};
      wallaby.testFramework.configure(jestConfig);
    },

    testFramework: "jest",

    debug: true
  };
};
