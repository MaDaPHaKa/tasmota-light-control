import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatDialog } from '@angular/material/dialog';
import { MatIconModule } from '@angular/material/icon';
import { AppStoreService } from '@services/app-store.service';
import { BulbsApiService } from '@services/bulbs-api.service';
import { Bulb, BulbInput, BulbTestResult } from '@model/bulb.model';
import { SnackbarService } from '@services/snackbar.service';
import { BulbFormComponent } from '../bulb-form/bulb-form.component';
import { BulbListComponent } from '../bulb-list/bulb-list.component';
import { ConfirmDialogComponent, ConfirmDialogData } from '@components/shared/confirm-dialog/confirm-dialog.component';

@Component({ selector:'app-bulbs-page', standalone:true, imports:[MatButtonModule,MatIconModule,BulbFormComponent,BulbListComponent], templateUrl:'./bulbs-page.component.html', styleUrl:'./bulbs-page.component.scss', changeDetection:ChangeDetectionStrategy.OnPush })
export class BulbsPageComponent {
  private readonly store=inject(AppStoreService); private readonly api=inject(BulbsApiService); private readonly dialog=inject(MatDialog); private readonly snackbar=inject(SnackbarService);
  protected readonly bulbs=this.store.bulbs; protected readonly statuses=this.store.statusByBulbId; protected readonly showForm=signal(false); protected readonly editing=signal<Bulb|null>(null); protected readonly pending=signal(false);
  protected toggleForm(){this.editing.set(null);this.showForm.update(v=>!v);}
  protected edit(bulb:Bulb){this.editing.set(bulb);this.showForm.set(true);}
  protected save(input:BulbInput){const request=this.editing()?this.api.replace(this.editing()!.id,input):this.api.create(input);this.pending.set(true);request.subscribe({next:b=>{this.store.upsertBulb(b);this.showForm.set(false);this.pending.set(false);this.snackbar.success(`${b.name} saved.`);},error:()=>{this.pending.set(false);this.snackbar.error('Could not save bulb.');}});}
  protected test(bulb:Bulb){this.api.testSaved(bulb.id).subscribe({next:r=>this.showTest(r),error:()=>this.snackbar.error('Bulb test failed.')});}
  protected testEndpoint(endpoint:{ipAddress:string;port:number}){this.api.testUnsaved(endpoint).subscribe({next:r=>this.showTest(r),error:()=>this.snackbar.error('Endpoint test failed.')});}
  private showTest(result:BulbTestResult){this.snackbar.info(result.message);}
  protected refresh(id:string){this.store.refreshStatus(id);}
  protected remove(bulb:Bulb){const data:ConfirmDialogData={title:`Delete ${bulb.name}?`,consequence:'This bulb registration will be permanently removed.',confirmLabel:'Delete bulb',destructive:true};this.dialog.open(ConfirmDialogComponent,{data}).afterClosed().subscribe(ok=>{if(!ok)return;this.api.delete(bulb.id).subscribe({next:()=>{this.store.removeBulb(bulb.id);this.snackbar.success('Bulb deleted.');},error:()=>this.snackbar.error('Could not delete bulb.')});});}
}
