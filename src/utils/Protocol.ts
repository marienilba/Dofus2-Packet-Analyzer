import {
  msg_from_id,
  types,
  types_from_id,
  primitives,
} from "./network-messages/2.63/messages";
import CustomDataWrapper from "./CustomDataWraper";
import BooleanByteWrapper from "./BooleanByteWrapper";
// @ts-ignore
import { Buffer } from "buffer";
import { ByteArray } from "./bytearray";

interface Anything {
  [key: string]: Message;
}

type Message = {
  name: string;
  parent: string | null;
  protocolId: number;
  vars: variable[];
  boolVars: variable[];
  hash_function: any;
};

export type variable = {
  name: string;
  length: string;
  type: string;
  optional: boolean;
};

export const PacketToObject = (data: CustomDataWrapper, packet_id: number) => {
  const msg = msg_from_id[packet_id];
  if (!msg) {
    console.log("Unvalid packet id, none message found");
    return;
  }
  return deserialize(data, msg.name);
};

export const getMsgFromId = msg_from_id as unknown as Anything;
export const getTypesFromName = types as unknown as Anything;
export const getTypeFromId = types_from_id as unknown as Anything;
export const getPrimitives = primitives as string[];

export function readAtomicType(data: CustomDataWrapper, desc: variable): {} {
  if (desc.optional) {
  }
  let result = {};
  try {
    if (desc.length) {
      let length = data.read(desc.length);
      if (typeof length != "number") {
        throw new Error("length not number : " + length);
      }
      let res = [];
      for (let i = 0; i < length; i++) {
        res.push(data.read(desc.type));
      }
      result = res;
    } else {
      result = data.read(desc.type);
    }
  } catch (ex) {
    console.log("error!", ex);
    return "ERROR";
  }

  return result;
}

export function deserialize(data: CustomDataWrapper, typeName: string): {} {
  let result = {};

  let msgSpec = getTypesFromName[typeName];

  if (!msgSpec) {
    throw new Error("msgSpec missing ! typeName: " + typeName);
  }

  //handle parent
  if (msgSpec.parent != null) {
    const res = deserialize(data, msgSpec.parent);
    result = { ...result, ...res };
  }

  //handle boolvars 8 by 8 (each boolvar is written in 1/8 byte)
  if (msgSpec.boolVars.length > 0) {
    for (let j = 0; j < msgSpec.boolVars.length; j += 8) {
      let _box0: number = data.readByte();
      for (let i = 0; i < 8 && i < msgSpec.boolVars.length / (j + 1); i++) {
        let bool1 = msgSpec?.boolVars[i];
        result = {
          ...result,
          [bool1.name]: BooleanByteWrapper.getFlag(_box0, i),
        };
      }
    }
  }

  //handle vars
  msgSpec.vars.map((v) => {
    if (getPrimitives.includes(v.type)) {
      let res = readAtomicType(data, v);
      result = { ...result, [v.name]: res };
    } else {
      if (v.length == null) {
        let type = v.type;
        if (type == "ID") {
          let id = data.readUnsignedShort();
          type = getTypeFromId[id]?.name;
          if (!type) console.log(id);
        }
        if (type) {
          result = { ...result, ...deserialize(data, type) };
        } else {
        }
      }
      // else if (v.type === "ID") {
      //   //
      // }
      else {
        const length = data.read(v.length);
        const res = [];
        for (let i = 0; i < length; i++) {
          let type = v.type;

          if (type === "ID") {
            let id = data.readUnsignedShort();

            type = types_from_id[id]?.name;
            if (!type) console.log(id);
          }
          if (type) {
            res.push(deserialize(data, type));
          }
        }
        result = { ...result, [v.name]: res };
      }
    }
  });
  return result;
}

// OSEF v

export function writeAtomicType(
  data: CustomDataWrapper,
  value: any,
  desc: variable
) {
  if (value == undefined) throw new Error("value undefined");
  let length: any = 1;
  if (desc.length) {
    length = value.length;
    data.write(desc.length, length);
    for (let i = 0; i < length; i++) {
      data.write(desc.type, value[i]);
    }
  } else {
    data.write(desc.type, value);
  }
}

export function serialize(
  dataWrapper: CustomDataWrapper = new CustomDataWrapper(new ByteArray()),
  data: any,
  typeName: string
): Buffer {
  if (!typeName || !data) {
    throw new Error("missing arg data: " + data + " typeName:" + typeName);
  }

  let msgSpec = getTypesFromName[typeName];

  if (!msgSpec) {
    throw new Error("no msgSpec for : " + typeName);
  }

  if (msgSpec.parent != null) {
    serialize(dataWrapper, data, msgSpec.parent);
  }

  //handle boolvars 8 by 8 (each boolvar is written in 1/8 byte)
  if (msgSpec.boolVars.length > 0) {
    for (let j = 0; j < msgSpec.boolVars.length; j += 8) {
      let _box0: number = 0;
      for (let i = 0; i < 8 && i < msgSpec.boolVars.length / (j + 1); i++) {
        _box0 = BooleanByteWrapper.setFlag(
          _box0,
          i,
          data[msgSpec?.boolVars[i].name]
        );
      }

      dataWrapper.writeByte(_box0);
    }
  }

  msgSpec.vars.map((desc) => {
    if (getPrimitives.includes(desc.type)) {
      writeAtomicType(dataWrapper, data[desc.name], desc);
    } else {
      if (desc.length == null) {
        if (!data[desc.name]) {
        }
        serialize(dataWrapper, data[desc.name], desc.type);
      } else {
        dataWrapper.write(desc.length, data[desc.name].length);
        for (let i = 0; i < data[desc.name].length; i++)
          serialize(dataWrapper, data[desc.name][i], desc.type);
      }
    }
  });
  return dataWrapper.getBuffer();
}
