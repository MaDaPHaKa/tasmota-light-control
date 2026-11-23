import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { BulbSettings, ResetSettings } from '@model/settings.model';
import { Uuid } from '@model/bulb.model';
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
  getBulbSettings(id: Uuid) {
    return this.http.get<BulbSettings>(`${environment.baseUrl}/api/v1/bulbs/${id}/settings`);
  }
  replaceBulbSettings(id: Uuid, settings: BulbSettings) {
    return this.http.put<BulbSettings>(`${environment.baseUrl}/api/v1/bulbs/${id}/settings`, settings);
  }
}
