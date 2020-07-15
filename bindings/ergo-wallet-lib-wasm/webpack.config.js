const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');

module.exports = {
  entry: './tests/index.js',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'index.js'
  },
  plugins: [
    new HtmlWebpackPlugin(),
    // Have this example work in Edge which doesn't ship `TextEncoder` or
    // `TextDecoder` at this time.
    new webpack.ProvidePlugin({
      TextDecoder: ['text-encoder', 'TextDecoder'],
      TextEncoder: ['text-encoder', 'TextEncoder']
    })
  ],
  mode: 'development',
  module: {
    rules: [
      {
        test: /test.*\.js$/,
        use: 'mocha-loader',
        exclude: /node_modules/
      }
    ]
  }
};
