import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { ResetSettings } from '@model/settings.model';
import { environment } from '../../environments/environment';
@Injectable({ providedIn: 'root' })
export class SettingsApiService {
  private readonly http = inject(HttpClient);
  private readonly url = `${environment.baseUrl}/api/v1/settings/reset`;
  getResetSettings() {
    return this.http.get<ResetSettings>(this.url);
  }
  replaceResetSettings(settings: ResetSettings) {
    return this.http.put<ResetSettings>(this.url, settings);
  }
}
