import { ApiFailure } from '@model/api-error.model';
export function formErrors(failure: ApiFailure): Record<string, string> {
  return {
    ...failure.fields,
    ...(Object.keys(failure.fields).length ? {} : { form: failure.message }),
  };
}
