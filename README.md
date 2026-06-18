# OpenAI Completions Proxy

A simple proxy web server that translates incoming requests to the legacy `/v1/completions` API to the newer `/v1/chat/completions` API, and forwards the request to an OpenAI-compatible server of your choice.

Set the server to forward to with the `OPENAI_BASE_URL` environment variable. You can also set an API key with `OPENAI_API_KEY`, or if you leave it unset, API keys in the request to the proxy will be forwarded to the upstream unmodified.

## License

This project's source code and documentation are licensed under the MIT license. See the [LICENSE](LICENSE) file for details.
