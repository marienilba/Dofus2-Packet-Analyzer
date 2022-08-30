"use strict";
// import { deflateSync, deflateRawSync, inflateSync, inflateRawSync } from "zlib";
// import { LZMA } from "lzma-native";
import { encodingExists, decode, encode } from "iconv-lite";
import { Buffer } from "buffer";

const CompressionAlgorithm = {
  DEFLATE: "deflate",
  LZMA: "lzma",
  ZLIB: "zlib",
} as const;

const Endian = { LITTLE_ENDIAN: "LE", BIG_ENDIAN: "BE" } as const;
const ObjectEncoding = { AMF0: 0, AMF3: 3 } as const;
// @ts-ignore
import { AMF0 } from "amf0-ts";
// @ts-ignore
import { AMF3 } from "amf3-ts";

/**
 * @description Helper function that converts data types to a buffer
 * @param {Buffer|Array|Number} v
 * @returns {Buffer}
 */
const convert = (v: Buffer | any[] | number): Buffer =>
  Buffer.isBuffer(v)
    ? v
    : Array.isArray(v)
    ? Buffer.from(v)
    : Number.isInteger(v)
    ? Buffer.alloc(v)
    : Buffer.alloc(0);

/**
 * @exports
 * @class
 */
export class ByteArray {
  /**
   * @private
   * @description The current position
   * @type {Number}
   */
  position: number;
  /**
   * @private
   * @description The byte order
   * @type {String}
   */
  endian: string;
  /**
   * @private
   * @description The object encoding
   * @type {Number}
   */
  objectEncoding: number;

  buffer: Buffer;
  /**
   * @constructor
   * @param {Buffer|Array|Number} buffer
   */
  constructor(buffer?: Buffer | any[] | number) {
    /**
     * @description Holds the data
     * @type {Buffer}
     */
    this.buffer = buffer ? convert(buffer) : Buffer.alloc(0);
    /**
     * @private
     * @description The current position
     * @type {Number}
     */
    this.position = 0;
    /**
     * @private
     * @description The byte order
     * @type {String}
     */
    this.endian = Endian.BIG_ENDIAN;
    /**
     * @private
     * @description The object encoding
     * @type {Number}
     */
    this.objectEncoding = ObjectEncoding.AMF3;
  }

  /**
   * @static
   * @description Registers a class alias
   * @param {Number} encoding
   * @param {String} aliasName
   * @param {ObjectEncoding} classObject
   */
  static registerClassAlias(encoding: any, aliasName: any, classObject: any) {
    if (encoding === ObjectEncoding.AMF0) {
      AMF0.registerClassAlias(aliasName, classObject);
    } else if (encoding === ObjectEncoding.AMF3) {
      AMF3.registerClassAlias(aliasName, classObject);
    } else {
      throw new Error(`Unknown object encoding: '${encoding}'.`);
    }
  }

  /**
   * @description Override for Object.prototype.toString.call
   * @returns {String}
   */
  get [Symbol.toStringTag]() {
    return "ByteArray";
  }

  /**
   * @description Returns the current position
   * @returns {Number}
   */
  // get position() {
  //   return this.position;
  // }

  /**
   * @description Sets the position
   * @param {Number} value
   */
  // set position(value) {
  //   if (value >= 0) {
  //     this.position = value;
  //   } else {
  //     throw new TypeError(`Invalid value for position: '${value}'.`);
  //   }
  // }

  /**
   * @description Returns the byte order
   * @returns {String}
   */
  // get endian() {
  //   return this.endian;
  // }

  /**
   * @description Sets the byte order
   * @param {String} value
   */
  // set endian(value: string) {
  //   if (value === "LE" || value === "BE") {
  //     this.endian = value;
  //   } else {
  //     throw new TypeError(`Invalid value for endian: '${value}'.`);
  //   }
  // }

  /**
   * @description Returns the object encoding
   * @returns {Number}
   */
  // get objectEncoding() {
  //   return this.objectEncoding;
  // }

  /**
   * @description Sets the object encoding
   * @param {Number} encoding
   */
  // set objectEncoding(encoding: number) {
  //   if (encoding === ObjectEncoding.AMF0 || encoding === ObjectEncoding.AMF3) {
  //     this.objectEncoding = encoding;
  //   } else {
  //     throw new Error(`Unknown object encoding: '${encoding}'.`);
  //   }
  // }

  /**
   * @description Returns the length of the buffer
   * @returns {Number}
   */
  get length() {
    return this.buffer.length;
  }

  /**
   * @description Sets the length of the buffer
   * @param {Number} value
   */
  set length(value) {
    if (!Number.isInteger(value) || value < 0) {
      throw new TypeError(`Invalid value for length: '${value}'.`);
    }

    if (value === 0) {
      this.clear();
    } else if (value !== this.length) {
      if (value < this.length) {
        this.buffer = this.buffer.slice(0, value);
        this.position = this.length;
      } else {
        this.expand(value);
      }
    }
  }

  /**
   * @description Returns the amount of bytes available
   * @returns {Number}
   */
  get bytesAvailable() {
    return this.length - this.position;
  }

  /**
   * @private
   * @description Reads a buffer function
   * @param {String} func
   * @param {Number} pos
   * @returns {Number}
   */
  private readBufferFunc(func: string, pos: number): number {
    // @ts-ignore
    const value = this.buffer[`${func}${this.endian}`](this.position);

    this.position += pos;

    return value;
  }

  /**
   * @private
   * @description Writes a buffer function
   * @param {Number} value
   * @param {String} func
   * @param {Number} pos
   */
  writeBufferFunc(value: number, func: string, pos: number) {
    this.expand(pos);

    // @ts-ignore
    this.buffer[`${func}${this.endian}`](value, this.position);
    this.position += pos;
  }

  /**
   * @private
   * @description Expands the buffer when needed
   * @param {Number} value
   */
  expand(value: number) {
    if (this.bytesAvailable < value) {
      const old = this.buffer;
      const size = old.length + (value - this.bytesAvailable);

      this.buffer = Buffer.alloc(size);
      old.copy(this.buffer);
    }
  }

  /**
   * @description Simulates signed overflow
   * @author truelossless
   * @param {Number} value
   * @param {Number} bits
   * @returns {Number}
   */
  signedOverflow(value: number, bits: number): number {
    const sign = 1 << (bits - 1);

    return (value & (sign - 1)) - (value & sign);
  }

  /**
   * @description Clears the buffer and sets the position to 0
   */
  clear() {
    this.buffer = Buffer.alloc(0);
    this.position = 0;
  }

  /**
   * @description Compresses the buffer
   * @param {String} algorithm
   */
  // async compress(algorithm: string = CompressionAlgorithm.ZLIB) {
  //   if (this.length === 0) {
  //     return;
  //   }

  //   algorithm = algorithm.toLowerCase();

  //   if (algorithm === CompressionAlgorithm.ZLIB) {
  //     this.buffer = deflateSync(this.buffer, { level: 9 });
  //   } else if (algorithm === CompressionAlgorithm.DEFLATE) {
  //     this.buffer = deflateRawSync(this.buffer);
  //   } else if (algorithm === CompressionAlgorithm.LZMA) {
  //     this.buffer = await LZMA().compress(this.buffer, 1);
  //   } else {
  //     throw new Error(`Invalid compression algorithm: '${algorithm}'.`);
  //   }

  //   this.position = this.length;
  // }

  /**
   * @description Reads a boolean
   * @returns {Boolean}
   */
  readBoolean() {
    return this.readByte() !== 0;
  }

  /**
   * @description Reads a signed byte
   * @returns {Number}
   */
  readByte() {
    return this.buffer.readInt8(this.position++);
  }

  /**
   * @description Reads multiple signed bytes from a ByteArray
   * @param {ByteArray} bytes
   * @param {Number} offset
   * @param {Number} length
   */
  readBytes(bytes: ByteArray, offset: number = 0, length: number = 0) {
    if (length === 0) {
      length = this.bytesAvailable;
    }

    if (length > this.bytesAvailable) {
      throw new RangeError("End of buffer was encountered.");
    }

    if (bytes.length < offset + length) {
      bytes.expand(offset + length);
    }

    for (let i = 0; i < length; i++) {
      bytes.buffer[i + offset] = this.buffer[i + this.position];
    }

    this.position += length;
  }

  /**
   * @description Reads a double
   * @returns {Number}
   */
  readDouble() {
    return this.readBufferFunc("readDouble", 8);
  }

  /**
   * @description Reads a float
   * @returns {Number}
   */
  readFloat() {
    return this.readBufferFunc("readFloat", 4);
  }

  /**
   * @description Reads a signed int
   * @returns {Number}
   */
  readInt() {
    return this.readBufferFunc("readInt32", 4);
  }

  /**
   * @description Reads a signed long
   * @returns {BigInt}
   */
  readLong() {
    // let n = this.readBufferFunc("readBigInt64", 8);
    // console.log(this.buffer.readBigUInt64BE().toString());
    let val = this.buffer.readBigUInt64BE();
    this.position += 8;

    return val;
  }

  /**
   * @description Reads a multibyte string
   * @param {Number} length
   * @param {String} charset
   * @returns {String}
   */
  readMultiByte(length: number, charset: string = "utf8"): string {
    const position = this.position;
    this.position += length;

    if (encodingExists(charset)) {
      const b = this.buffer.slice(position, this.position);
      const stripBOM =
        (charset === "utf8" || charset === "utf-8") &&
        b.length >= 3 &&
        b[0] === 0xef &&
        b[1] === 0xbb &&
        b[2] === 0xbf;
      const value = decode(b, charset, { stripBOM });

      stripBOM ? (length -= 3) : 0;

      if (Buffer.byteLength(value) !== length) {
        throw new RangeError("End of buffer was encountered.");
      }

      return value;
    } else {
      throw new Error(`Invalid character set: '${charset}'.`);
    }
  }

  /**
   * @description Reads an object
   * @returns {Object}
   */
  readObject() {
    const [position, value] =
      this.objectEncoding === ObjectEncoding.AMF0
        ? AMF0.parse(this.buffer, this.position)
        : AMF3.parse(this.buffer, this.position);

    this.position += position;

    return value;
  }

  /**
   * @description Reads a signed short
   * @returns {Number}
   */
  readShort() {
    return this.readBufferFunc("readInt16", 2);
  }

  /**
   * @description Reads an unsigned byte
   * @returns {Number}
   */
  readUnsignedByte() {
    return this.buffer.readUInt8(this.position++);
  }

  /**
   * @description Reads an unsigned int
   * @returns {Number}
   */
  readUnsignedInt() {
    return this.readBufferFunc("readUInt32", 4);
  }

  /**
   * @description Reads an unsigned short
   * @returns {Number}
   */
  readUnsignedShort() {
    return this.readBufferFunc("readUInt16", 2);
  }

  /**
   * @description Reads an unsigned long
   * @returns {BigInt}
   */
  readUnsignedLong() {
    let val = this.buffer.readBigUInt64BE();
    this.position += 8;
    return val;
  }

  /**
   * @description Reads a UTF-8 string
   * @returns {String}
   */
  readUTF() {
    return this.readMultiByte(this.readUnsignedShort());
  }

  /**
   * @description Reads UTF-8 bytes
   * @param {Number} length
   * @returns {String}
   */
  readUTFBytes(length: number): string {
    return this.readMultiByte(length);
  }

  /**
   * @description Converts the buffer to JSON
   * @returns {Object}
   */
  toJSON() {
    return Object.assign({}, this.buffer.toJSON().data);
  }

  /**
   * @description Converts the buffer to a string
   * @param {String} charset
   * @returns {String}
   */
  // toString(charset = "utf8") {
  //   if (encodingExists(charset)) {
  //     return decode(this.buffer, charset);
  //   } else {
  //     throw new Error(`Invalid character set: '${charset}'.`);
  //   }
  // }

  /**
   * @description Decompresses the buffer
   * @param {String} algorithm
   */
  // async uncompress(algorithm: string = CompressionAlgorithm.ZLIB) {
  //   if (this.length === 0) {
  //     return;
  //   }

  //   algorithm = algorithm.toLowerCase();

  //   if (algorithm === CompressionAlgorithm.ZLIB) {
  //     this.buffer = inflateSync(this.buffer, { level: 9 });
  //   } else if (algorithm === CompressionAlgorithm.DEFLATE) {
  //     this.buffer = inflateRawSync(this.buffer);
  //   } else if (algorithm === CompressionAlgorithm.LZMA) {
  //     this.buffer = await LZMA().decompress(this.buffer);
  //   } else {
  //     throw new Error(`Invalid decompression algorithm: '${algorithm}'.`);
  //   }

  //   this.position = 0;
  // }

  /**
   * @description Writes a boolean
   * @param {Boolean} value
   */
  writeBoolean(value: boolean) {
    this.writeByte(value ? 1 : 0);
  }

  /**
   * @description Writes a signed byte
   * @param {Number} value
   */
  writeByte(value: number) {
    this.expand(1);
    this.buffer.writeInt8(this.signedOverflow(value, 8), this.position++);
  }

  /**
   * @description Writes multiple signed bytes to a ByteArray
   * @param {ByteArray} bytes
   * @param {Number} offset
   * @param {Number} length
   */
  writeBytes(bytes: ByteArray, offset: number = 0, length: number = 0) {
    if (length === 0) {
      length = bytes.length - offset;
    }

    this.expand(length);

    for (let i = 0; i < length; i++) {
      this.buffer[i + this.position] = bytes.buffer[i + offset];
    }

    this.position += length;
  }

  /**
   * @description Writes a double
   * @param {Number} value
   */
  writeDouble(value: number) {
    this.writeBufferFunc(value, "writeDouble", 8);
  }

  /**
   * @description Writes a float
   * @param {Number} value
   */
  writeFloat(value: number) {
    this.writeBufferFunc(value, "writeFloat", 4);
  }

  /**
   * @description Writes a signed int
   * @param {Number} value
   */
  writeInt(value: number) {
    this.writeBufferFunc(this.signedOverflow(value, 32), "writeInt32", 4);
  }

  /**
   * @description Writes a signed long
   * @param {BigInt} value
   */
  writeLong(value: number) {
    this.writeBufferFunc(value, "writeBigInt64", 8);
  }

  /**
   * @description Writes a multibyte string
   * @param {String} value
   * @param {String} charset
   */
  writeMultiByte(value: string, charset: string = "utf8") {
    this.position += Buffer.byteLength(value);

    if (encodingExists(charset)) {
      this.buffer = Buffer.concat([this.buffer, encode(value, charset)]);
    } else {
      throw new Error(`Invalid character set: '${charset}'.`);
    }
  }

  /**
   * @description Writes an object
   * @param {Object} value
   */
  writeObject(value: Object) {
    const bytes =
      this.objectEncoding === ObjectEncoding.AMF0
        ? AMF0.stringify(value)
        : AMF3.stringify(value);

    this.position += bytes.length;
    this.buffer = Buffer.concat([this.buffer, Buffer.from(bytes)]);
  }

  /**
   * @description Writes a signed short
   * @param {Number} value
   */
  writeShort(value: number) {
    this.writeBufferFunc(this.signedOverflow(value, 16), "writeInt16", 2);
  }

  /**
   * @description Writes an unsigned byte
   * @param {Number} value
   */
  writeUnsignedByte(value: number) {
    this.expand(1);
    this.buffer.writeUInt8(value, this.position++);
  }

  /**
   * @description Writes an unsigned int
   * @param {Number} value
   */
  writeUnsignedInt(value: number) {
    this.writeBufferFunc(value, "writeUInt32", 4);
  }

  /**
   * @description Writes an unsigned short
   * @param {Number} value
   */
  writeUnsignedShort(value: number) {
    this.writeBufferFunc(value, "writeUInt16", 2);
  }

  /**
   * @description Writes an unsigned long
   * @param {BigInt} value
   */
  writeUnsignedLong(value: number) {
    this.writeBufferFunc(value, "writeBigUInt64", 8);
  }

  /**
   * @description Writes a UTF-8 string
   * @param {String} value
   */
  writeUTF(value: string) {
    this.writeUnsignedShort(Buffer.byteLength(value));
    this.writeMultiByte(value);
  }

  /**
   * @description Writes UTF-8 bytes
   * @param {String} value
   */
  writeUTFBytes(value: string) {
    this.writeMultiByte(value);
  }
}
