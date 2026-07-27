import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import {
  Bulb,
  BulbEndpointInput,
  BulbInput,
  BulbTestResult,
  LiveBulbState,
  Uuid,
} from '@model/bulb.model';
import { environment } from '../../environments/environment';
@Injectable({ providedIn: 'root' })
export class BulbsApiService {
  private readonly http = inject(HttpClient);
  private readonly base = `${environment.baseUrl}/api/v1/bulbs`;
  list() {
    return this.http.get<Bulb[]>(this.base);
  }
  get(id: Uuid) {
    return this.http.get<Bulb>(`${this.base}/${id}`);
  }
  create(input: BulbInput) {
    return this.http.post<Bulb>(this.base, input);
  }
  replace(id: Uuid, input: BulbInput) {
    return this.http.put<Bulb>(`${this.base}/${id}`, input);
  }
  delete(id: Uuid) {
    return this.http.delete<void>(`${this.base}/${id}`);
  }
  testUnsaved(endpoint: BulbEndpointInput) {
    return this.http.post<BulbTestResult>(`${this.base}/test`, endpoint);
  }
  testSaved(id: Uuid) {
    return this.http.post<BulbTestResult>(`${this.base}/${id}/test`, {});
  }
  getAllStatuses() {
    return this.http.get<LiveBulbState[]>(`${this.base}/status`);
  }
  getStatus(id: Uuid) {
    return this.http.get<LiveBulbState>(`${this.base}/${id}/status`);
  }
}
