import { open, save } from "@tauri-apps/plugin-dialog";

type Filter = { name: string; extensions: string[] };

export async function pickFile(title: string, filters?: Filter[]): Promise<string | null> {
  const r = await open({ title, multiple: false, directory: false, filters });
  return typeof r === "string" ? r : null;
}

export async function pickFolder(title: string): Promise<string | null> {
  const r = await open({ title, multiple: false, directory: true });
  return typeof r === "string" ? r : null;
}

export async function saveFile(title: string, defaultPath: string, filters?: Filter[]): Promise<string | null> {
  return (await save({ title, defaultPath, filters })) ?? null;
}

export const F = {
  zip: { name: "ZIP archive", extensions: ["zip"] },
  main: { name: "Save file (main.dat)", extensions: ["dat"] },
  png: { name: "PNG image", extensions: ["png"] },
  jpg: { name: "JPEG image", extensions: ["jpg", "jpeg"] },
  nhi: { name: "Item list (.nhi)", extensions: ["nhi"] },
  nhv: { name: "Villager (.nhv2, .nhv)", extensions: ["nhv2", "nhv"] },
  nhvh: { name: "Villager house (.nhvh2, .nhvh)", extensions: ["nhvh2", "nhvh"] },
  nhph: { name: "Player house (.nhph2, .nhph)", extensions: ["nhph2", "nhph"] },
  nhpr: { name: "Room (.nhpr2, .nhpr)", extensions: ["nhpr2", "nhpr"] },
  nhd: { name: "Design (.nhd)", extensions: ["nhd"] },
  nhpd: { name: "PRO design (.nhpd)", extensions: ["nhpd"] },
  nhpb: { name: "Letter (.nhpb)", extensions: ["nhpb"] },
  nhl: { name: "Field items (.nhl)", extensions: ["nhl"] },
  nhlf: { name: "Field item flags (.nhlf)", extensions: ["nhlf"] },
  nht: { name: "Terrain (.nht)", extensions: ["nht"] },
  nhb: { name: "Buildings (.nhb)", extensions: ["nhb"] },
  nha: { name: "Acres (.nha)", extensions: ["nha"] },
  nhmd: { name: "Ground designs (.nhmd)", extensions: ["nhmd"] },
  nhbp: { name: "Bulletin post (.nhbp)", extensions: ["nhbp"] },
  txt: { name: "Text", extensions: ["txt"] },
  all: { name: "All files", extensions: ["*"] },
};
