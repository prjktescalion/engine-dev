export interface TransformComponent {
  type: 'transform';
  x: number;
  y: number;
  scaleX: number;
  scaleY: number;
  rotation: number;
}

export interface SpriteComponent {
  type: 'sprite';
  assetPath: string;
  dataUrl: string;
}

export interface ScriptComponent {
  type: 'script';
  filePath: string;
  lang: 'rust' | 'java' | 'python';
}

export type Component = TransformComponent | SpriteComponent | ScriptComponent;

export interface Entity {
  id: string;
  name: string;
  components: Component[];
}

export interface Asset {
  id: string;
  name: string;
  path: string;
  kind: 'image' | 'script' | 'audio' | 'other' | 'dir';
  is_dir: boolean;
  children: Asset[];
}

export interface EditorSettings {
  editor: 'vscode' | 'jetbrains' | 'custom';
  custom_path: string;
  theme: 'dark' | 'light';
}

export type Tool = 'select' | 'move' | 'scale' | 'rotate';
