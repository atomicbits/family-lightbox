// Lightroom's edited versions of a photo (F025/S04, data contract 4 and 5.5): which one shows,
// and how each is named in the window.

/** The chosen version, as the merge rule says: the mark when it names one that is here (or the
 * plain original), else the latest edit, else the plain original. */
export function chosenVariant(variants: string[] | undefined, mark: string | undefined): string {
  const here = variants ?? [];
  if (mark !== undefined && (mark === "" || here.includes(mark))) return mark;
  return here[0] ?? "";
}

/** "Origineel", "Bewerkt", or a virtual copy's name from its slug: "Zwart wit". */
export function variantLabel(variant: string): string {
  if (variant === "") return "Origineel";
  if (variant === "lr") return "Bewerkt";
  const name = variant.slice(3).replace(/-/g, " ");
  return name.charAt(0).toUpperCase() + name.slice(1);
}

/** All versions of a photo in the order of the switcher: the plain original first, then the edits. */
export function allVariants(variants: string[] | undefined): string[] {
  return ["", ...(variants ?? [])];
}
