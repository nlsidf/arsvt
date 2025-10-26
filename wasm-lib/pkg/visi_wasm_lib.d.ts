/* tslint:disable */
/* eslint-disable */
export class MediaConverter {
  free(): void;
  constructor();
  /**
   * 将文件字节数据转换为PNG图像数据URL
   */
  static file_bytes_to_image_data_url(file_bytes: Uint8Array, format: string): string;
  /**
   * 从图像数据URL提取文件字节数据
   */
  static image_data_url_to_file_bytes(data_url: string): Uint8Array;
  /**
   * 将文件字节数据转换为WAV音频数据URL
   */
  static file_bytes_to_audio_data_url(file_bytes: Uint8Array): string;
  /**
   * 从音频数据URL提取文件字节数据
   */
  static audio_data_url_to_file_bytes(data_url: string): Uint8Array;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_mediaconverter_free: (a: number, b: number) => void;
  readonly mediaconverter_new: () => number;
  readonly mediaconverter_file_bytes_to_image_data_url: (a: number, b: number, c: number, d: number) => [number, number];
  readonly mediaconverter_image_data_url_to_file_bytes: (a: number, b: number) => [number, number];
  readonly mediaconverter_file_bytes_to_audio_data_url: (a: number, b: number) => [number, number];
  readonly mediaconverter_audio_data_url_to_file_bytes: (a: number, b: number) => [number, number];
  readonly __wbindgen_export_0: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
