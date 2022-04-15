# kcp-ts

TypeScript port of the [KCP protocol][2] modified to support Genshin Impact.

Genshin Impact KCP adds an unsigned 32-bit integer field between `conv` and `cmd` in the
packet headers which makes it incompatible with the [original KCP protocol][3].
This implementation adds built-in support for that field.

## Dependencies

- [denque][1]

## API

API should be mostly self-explanatory if you are familiar with KCP already.
The entire implementation is contained within a single file [kcp.ts](kcp.ts).

## License

kcp-ts is licensed under the [MIT License](LICENSE).

[1]: https://www.npmjs.com/package/denque
[2]: https://github.com/skywind3000/kcp
[3]: https://github.com/skywind3000/kcp/blob/master/protocol.txt
