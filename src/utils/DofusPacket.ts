export interface DofusPacket {
  source: string;
  time: string;
  id: string;
  name: string;
  raw: string;
  body: { [key: string]: any };
}
