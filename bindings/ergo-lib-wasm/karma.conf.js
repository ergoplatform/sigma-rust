const webpack = require("webpack");
const outputDir = __dirname + "/dist";

module.exports = function (config) {
  config.set({
    frameworks: ["mocha", "chai", "webpack"],
    files: [
      "tests/**/*.js",
      "tests_browser/**/*.js",
      { pattern: `${outputDir}/*.wasm`, included: false, served: true },
    ],
    client: {
      mocha: {
        timeout: 900000
      }
    },
    webpack: {
      mode: "development",
      module: {
        rules: [
          {
            test: /\.js$/,
            loader: "babel-loader",
            options: {
              babelrc: false,
            },
            exclude: /node_modules/,
          },
        ],
      },
      resolve: {
        extensions: [".ts", ".js"],
        fallback: {
          buffer: require.resolve("buffer/"),
        },
      },
      experiments: {
        asyncWebAssembly: true,
      },
      plugins: [
        new webpack.ProvidePlugin({
          Buffer: ["buffer", "Buffer"],
        }),
      ],
      output: {
        path: outputDir,
      },
    },
    webpackMiddleware: {
      stats: "error-only",
    },
    preprocessors: {
      "tests/**/*.js": ["webpack"],
      "tests_browser/**/*.js": ["webpack"],
    },
    reporters: ["spec"],
    port: 9876,
    logLevel: config.LOG_INFO,

    browsers: ["ChromeHeadless"],
    browserNoActivityTimeout: 900000,

    autoWatch: false,
    singleRun: true,
    concurrency: Infinity,
  });
};
