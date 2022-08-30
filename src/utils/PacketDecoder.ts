import { ByteArray } from "./bytearray";
import CustomDataWrapper from "./CustomDataWraper";
import { msg_from_id } from "./network-messages/2.63/messages";
import { PacketToObject } from "./Protocol";
// @ts-ignore
import { Buffer } from "buffer";

export interface DofusPacket {
  source: string;
  time: string;
  id: number;
  name: string;
  raw: string;
  body: any;
}

export default class PacketDecoder {
  // dofus packets that are too long to be read in one tcp packet are stored here
  private static sba: CustomDataWrapper = new CustomDataWrapper(
    new ByteArray()
  );
  private static splitPacket = false;
  private static splitPacketId: number;
  private static splitPacketLength: number;
  private static splitPacketPort: number;

  private static packetQueue: DofusPacket[] = [];

  public decodeTcp(tcpContent: number[], port: number) {
    const ba = new CustomDataWrapper(new ByteArray(tcpContent));
    while (ba.bytesAvailable > 0) {
      if (PacketDecoder.splitPacket) {
        console.log(
          `We're split, at ${PacketDecoder.sba.length} + ${ba._data.bytesAvailable}, looking for ${PacketDecoder.splitPacketLength}`
        );

        if (
          PacketDecoder.sba.length + ba._data.bytesAvailable <
          PacketDecoder.splitPacketLength
        ) {
          ba.readBytes(
            PacketDecoder.sba._data,
            PacketDecoder.sba.length,
            ba.bytesAvailable
          );
        } else {
          ba.readBytes(
            PacketDecoder.sba._data,
            PacketDecoder.sba.length,
            PacketDecoder.splitPacketLength - PacketDecoder.sba.length
          );
          PacketDecoder.sba.setPosition(0);

          PacketDecoder.decodeDofusMessage(
            PacketDecoder.sba,
            PacketDecoder.splitPacketId,
            PacketDecoder.splitPacketLength,
            PacketDecoder.splitPacketPort
          );

          PacketDecoder.resetSplit();
        }
      } else {
        if (ba.bytesAvailable < 2) {
          console.log("Empty packet !");
          return;
        }

        let hiHeader = ba.readUnsignedShort();
        let packetId = hiHeader >> 2;
        let lengthType = hiHeader & 0b11;
        let length = 0;
        let instanceId = -1;

        if (port !== 5555) {
          instanceId = ba.readUnsignedInt();
        }

        if (lengthType === 0) {
          length = 0;
        } else if (lengthType === 1) {
          length = ba.readUnsignedByte();
        } else if (lengthType === 2) {
          length = ba.readUnsignedShort();
        } else if (lengthType === 3) {
          length =
            ((ba.readByte() & 255) << 16) +
            ((ba.readByte() & 255) << 8) +
            (ba.readByte() & 255);
        }

        // console.log("length | available | packetId");
        // console.log(
        //   length +
        //     " | " +
        //     ba.bytesAvailable +
        //     " | " +
        //     msg_from_id[packetId]?.name
        // );
        if (msg_from_id[packetId] === undefined) return;
        if (length > ba.bytesAvailable) {
          PacketDecoder.splitPacket = true;
          PacketDecoder.splitPacketPort = port;
          PacketDecoder.splitPacketLength = length;
          PacketDecoder.splitPacketId = packetId;

          ba.readBytes(
            PacketDecoder.sba._data,
            PacketDecoder.sba._data.length,
            ba._data.bytesAvailable
          );
        } else {
          PacketDecoder.decodeDofusMessage(ba, packetId, length, port);
        }
      }
    }
  }

  private static decodeDofusMessage(
    packetContent: CustomDataWrapper,
    packetId: number,
    length: number,
    port: number
  ) {
    const initialPos = packetContent.getPosition();

    let messageObject = PacketToObject(packetContent, packetId);

    const consumed = packetContent.getPosition() - initialPos;
    if (length - consumed !== 0) {
      console.log("warning: forced to trim a packet !");
      packetContent.setPosition(initialPos + length);
    }

    // console.log({
    //   source: port === 443 || port === 5555 ? "Server" : "Client",
    //   time: PacketDecoder.formatTimestamp(),
    //   id: packetId,
    //   name: msg_from_id[packetId]?.name || "Unknown",
    //   raw: packetContent._data.toJSON(),
    //   body: messageObject,
    // });
    this.packetQueue.push({
      source: port === 443 || port === 5555 ? "Server" : "Client",
      time: PacketDecoder.formatTimestamp(),
      id: packetId,
      name: msg_from_id[packetId]?.name || "Unknown",
      raw: packetContent._data.toJSON().toString(),
      body: messageObject,
    });
  }

  public getQueue(): DofusPacket[] {
    const q = PacketDecoder.packetQueue;
    PacketDecoder.packetQueue = [];
    return q;
  }

  public pollDofusPacket(): DofusPacket | null {
    if (PacketDecoder.packetQueue.length === 0) {
      return null;
    } else {
      return PacketDecoder.packetQueue.pop();
    }
  }

  private static formatTimestamp(): string {
    let date = new Date();
    let hh = date.getHours();
    let mm = date.getMinutes();
    let ss = date.getSeconds();

    let hhS: string = hh.toString();
    let mmS: string = mm.toString();
    let ssS: string = ss.toString();

    if (hh < 10) {
      hhS = "0" + hh;
    }
    if (mm < 10) {
      mmS = "0" + mm;
    }
    if (ss < 10) {
      ssS = "0" + ss;
    }
    return hhS + ":" + mmS + ":" + ssS;
  }

  static resetSplit() {
    PacketDecoder.sba = new CustomDataWrapper(new ByteArray());
    PacketDecoder.splitPacket = false;
    PacketDecoder.splitPacketLength = 0;
    PacketDecoder.splitPacketId = 0;
    PacketDecoder.splitPacketPort = 0;
  }
}
