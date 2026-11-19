// The vehicle's picture: loaded from a file the player picks, shrunk to a
// sensible size, and kept in the design as a data URL so it travels with
// the .bav file.

/** The longest side a kept picture has, in pixels. */
export const MAX_SIDE = 1000;

/** A picture's size shrunk to fit `max` on its longest side (never grown). */
export function fitSize(width: number, height: number, max = MAX_SIDE): { width: number; height: number } {
  const scale = Math.min(1, max / Math.max(width, height));
  return { width: Math.max(1, Math.round(width * scale)), height: Math.max(1, Math.round(height * scale)) };
}

/**
 * The picture in `file` as a data URL, shrunk to fit: a PNG stays a PNG
 * (keeping its transparency), anything else becomes a JPEG.
 */
export async function loadPicture(file: File): Promise<string> {
  const bitmap = await createImageBitmap(file);
  const { width, height } = fitSize(bitmap.width, bitmap.height);
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("This window can't draw pictures.");
  const png = file.type === "image/png";
  if (!png) {
    // A JPEG has no transparency: white behind what was see-through.
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, width, height);
  }
  ctx.drawImage(bitmap, 0, 0, width, height);
  bitmap.close();
  return png ? canvas.toDataURL("image/png") : canvas.toDataURL("image/jpeg", 0.85);
}

/** Whether a design's picture is one the window shows (as Rust checks it). */
export function isPicture(image: string): boolean {
  return image.startsWith("data:image/png;base64,") || image.startsWith("data:image/jpeg;base64,");
}
