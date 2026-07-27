import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import {
  ApplyProfileRequest,
  ControlOperationResult,
  DirectLinkRequest,
  ResetBulbsRequest,
} from '@model/control.model';
import { environment } from '../../environments/environment';
@Injectable({ providedIn: 'root' })
export class ControlApiService {
  private readonly http = inject(HttpClient);
  private readonly base = `${environment.baseUrl}/api/v1`;
  applyProfile(request: ApplyProfileRequest) {
    return this.http.post<ControlOperationResult>(`${this.base}/actions/apply-profile`, request);
  }
  reset(request: ResetBulbsRequest) {
    return this.http.post<ControlOperationResult>(`${this.base}/actions/reset`, request);
  }
  resetAll() {
    return this.http.post<ControlOperationResult>(`${this.base}/actions/reset-all`, null);
  }
  generateDirectLink(request: DirectLinkRequest) {
    return this.http.post(`${this.base}/direct-links`, request, { responseType: 'text' });
  }
}
