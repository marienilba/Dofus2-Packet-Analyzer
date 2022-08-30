// @ts-ignore
import { Buffer } from "buffer";
import { ByteArray } from "./bytearray";

export default class CustomDataWrapper {
  private static INT_SIZE: number = 32;

  private static SHORT_SIZE: number = 16;

  private static SHORT_MIN_VALUE: number = -32768;

  private static SHORT_MAX_VALUE: number = 32767;

  private static UNSIGNED_SHORT_MAX_VALUE: number = 65536;

  private static CHUNCK_BIT_SIZE: number = 7;

  private static MAX_ENCODING_LENGTH: number = Math.ceil(
    CustomDataWrapper.INT_SIZE / CustomDataWrapper.CHUNCK_BIT_SIZE
  );

  private static MASK_10000000: number = 128;

  private static MASK_01111111: number = 127;

  public _data: ByteArray;

  constructor(data: ByteArray, position: number = 0) {
    this._data = data;
    this._data.position = position;
  }

  public getPosition = () => {
    return this._data.position;
  };
  public setPosition = (pos: number) => {
    this._data.position = pos;
  };

  public getBuffer = (): Buffer => {
    return this._data.buffer;
  };

  public read(type: string) {
    switch (type) {
      case "UnsignedByte":
        return this.readUnsignedByte();
      case "Byte":
        return this.readByte();
      case "UnsignedShort":
        return this.readUnsignedShort();
      case "Short":
        return this.readShort();
      case "Int":
        return this.readInt();
      case "Boolean":
        return this.readBoolean();
      case "UTF":
        return this.readUTF();
      case "Double":
        return this.readDouble();
      case "VarUhLong":
        return this.readVarLong();
      case "VarLong":
        return this.readVarLong();
      case "VarUhInt":
        return this.readVarUhInt();
      case "VarInt":
        return this.readVarInt();
      case "VarShort":
        return this.readVarShort();
      case "VarUhShort":
        return this.readVarUhShort();

      default:
        throw new Error(`${type} not implemented`);
    }
  }

  public write(type: string, value: any) {
    switch (type) {
      case "UnsignedShort":
        return this.writeUnsignedShort(value);
      case "Short":
        return this.writeShort(value);
      case "VarUhLong":
      case "VarLong":
        return this.writeVarLong2(value);
      case "VarUhInt":
      case "VarInt":
        return this.writeVarInt(value);
      case "Boolean":
        return this.writeBoolean(value);
      case "UTF":
        return this.writeUTF(value);
      case "VarShort":
        return this.writeVarShort(value);
      case "VarUhShort":
        return this.writeVarShort(value);
      case "Int":
        return this.writeInt(value);
      case "Byte":
        return this.writeByte(value);
      case "Double":
        return this.writeDouble(value);

      default:
        throw new Error(`${type} not implemented`);
    }
  }
  readVarLong() {
    return this._data.readLong();
  }

  public readShort(): number {
    return this._data.readShort();
  }

  public readVarInt(): number {
    var b = 0;
    var value = 0;
    var offset = 0;
    var hasNext = false;
    while (offset < 32) {
      b = this._data.readByte();
      hasNext = (b & 128) == 128;
      if (offset > 0) {
        value = value + ((b & 127) << offset);
      } else {
        value = value + (b & 127);
      }
      offset = offset + 7;
      if (!hasNext) {
        return value;
      }
    }
    throw new Error("Too much data");
  }

  public readVarUhInt(): number {
    return this.readVarInt();
  }

  public readVarShort(): number {
    var b = 0;
    var value = 0;
    var offset = 0;
    var hasNext = false;
    while (offset < 16) {
      b = this._data.readByte();
      hasNext = (b & 128) == 128;
      if (offset > 0) {
        value = value + ((b & 127) << offset);
      } else {
        value = value + (b & 127);
      }
      offset = offset + 7;
      if (!hasNext) {
        if (value > 32767) {
          value = value - 65536;
        }
        return value;
      }
    }
    throw new Error("Too much data");
  }

  public readVarUhShort(): number {
    return this.readVarShort();
  }

  public readByte(): number {
    return this._data.readByte();
  }

  public readDouble(): number {
    return this._data.readDouble();
  }

  public readUTF(): string {
    return this._data.readUTF();
  }

  public readUnsignedShort(): number {
    return this._data.readUnsignedShort();
  }

  public readInt(): number {
    return this._data.readInt();
  }

  public readBoolean(): boolean {
    return this._data.readByte() != 0;
  }

  public writeUnsignedShort(value: number): void {
    this._data.writeUnsignedShort(value);
  }
  public writeByte(value: number): void {
    this._data.writeByte(value);
  }

  public writeBytes(value: Buffer): void {
    this._data.writeBytes(new ByteArray(value));
  }

  public writeVarInt(value: number): void {
    if (value >= 0 && value <= CustomDataWrapper.MASK_01111111) {
      this._data.writeByte(value);
      return;
    }
    var b: number = 0;
    var c: number = value;
    for (; c != 0; ) {
      b = c & CustomDataWrapper.MASK_01111111;
      c = c >>> CustomDataWrapper.CHUNCK_BIT_SIZE;
      if (c > 0) {
        b = b | CustomDataWrapper.MASK_10000000;
      }
      this._data.writeByte(b);
    }
  }

  public writeVarLong2(val: number) {
    return this._data.writeLong(val);
  }

  public writeVarShort(value: number): void {
    return this._data.writeShort(value);
  }

  public writeShort(value: number): void {
    this._data.writeShort(value);
  }
  public writeUTF(value: string): void {
    this._data.writeUTF(value);
  }

  public writeInt(value: number): void {
    this._data.writeInt(value);
  }
  public writeInt32(value: number): void {
    this._data.writeShort(value);
  }
  public writeBoolean(value: boolean): void {
    this._data.writeBoolean(value);
  }

  public writeDouble(value: number): void {
    this._data.writeDouble(value);
  }

  public readUnsignedByte(): number {
    return this._data.readUnsignedByte();
  }

  private readInt64(): bigint {
    return this._data.readLong();
  }

  public writeVarLong(value: number): void {
    this._data.writeLong(value);
  }

  public readUnsignedInt(): number {
    return this._data.readUnsignedInt();
  }

  public readFloat(): number {
    return this._data.readFloat();
  }

  public writeUnsignedInt(value: number) {
    this._data.writeUnsignedInt(value);
  }

  private readUInt64() {
    return this._data.readUnsignedLong();
  }

  public get bytesAvailable(): number {
    return this._data.bytesAvailable;
  }

  public readBytes(bytes: ByteArray, offset?: number, length?: number) {
    // this._data.readBytes(bytes, offset, length);
    this._data.readBytes(bytes, offset, length);
  }

  public get length(): number {
    return this._data.length;
  }
}
