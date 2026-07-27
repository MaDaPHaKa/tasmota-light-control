export interface ValidationError {
  kind: string;
  message: string;
}
export function validateTrimmedRequired(
  value: string,
  maxLength: number,
): ValidationError | undefined {
  const trimmed = value.trim();
  if (!trimmed) return { kind: 'required', message: 'Value is required.' };
  if (trimmed.length > maxLength)
    return { kind: 'maxLength', message: `Use ${maxLength} characters or fewer.` };
  return undefined;
}
export function validateCanonicalIpv4(value: string): ValidationError | undefined {
  if (!/^((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.|$)){4}$/.test(value))
    return { kind: 'ipv4', message: 'Enter a valid IPv4 address.' };
  return undefined;
}
export function validateIntegerRange(
  value: number,
  min: number,
  max: number,
): ValidationError | undefined {
  if (!Number.isInteger(value) || value < min || value > max)
    return { kind: 'range', message: `Enter an integer from ${min} to ${max}.` };
  return undefined;
}
export function validateUppercaseRgbHex(value: string): ValidationError | undefined {
  if (!/^#[0-9A-F]{6}$/.test(value))
    return { kind: 'rgb', message: 'Enter uppercase RGB hex, for example #88C0D0.' };
  return undefined;
}
