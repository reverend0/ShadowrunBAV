// Closing the window: with unsaved changes, the player is asked first, as
// in a word processor (Save, Don't Save, Cancel).

import { fileName } from "./format.ts";

export const SAVE = "Save";
export const DONT_SAVE = "Don’t Save";
export const CANCEL = "Cancel";

/** What closing needs from the app. */
export interface Closing {
  dirty: boolean;
  path: string | null;
  /** Asks with the three buttons; gives the label clicked. */
  choose: (text: string) => Promise<string>;
  /** Saves (Save As for an untitled design); false if it didn't happen. */
  save: () => Promise<boolean>;
}

/** The design as the question names it: “Raptor.bav”, or this design. */
export function designName(path: string | null): string {
  return path ? `“${fileName(path)}”` : "this design";
}

/** Whether the window may close now. */
export async function mayClose(app: Closing): Promise<boolean> {
  if (!app.dirty) return true;
  const answer = await app.choose(`Save your changes to ${designName(app.path)} before closing?`);
  if (answer === SAVE) return app.save();
  return answer === DONT_SAVE; // Cancel, or the dialog closed some other way, keeps it open
}
