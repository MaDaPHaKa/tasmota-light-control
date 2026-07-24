import { ChangeDetectionStrategy, Component, ElementRef, DestroyRef, inject, signal, viewChild, viewChildren } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { finalize } from 'rxjs';
import { FormField, form, required, validate } from '@angular/forms/signals';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatRadioModule } from '@angular/material/radio';
import { ApiFailure } from '@model/api-error.model';
import { ResetSettings } from '@model/settings.model';
import { validateIntegerRange, validateUppercaseRgbHex } from '@functions/validators.function';
import { SettingsApiService } from '@services/settings-api.service';
import { ModePreviewComponent } from '@components/shared/mode-preview/mode-preview.component';
import { RgbColorPickerComponent } from '@components/shared/rgb-color-picker/rgb-color-picker.component';
import { SnackbarService } from '@services/snackbar.service';

type SettingsField='dimmer'|'mode'|'rgbColor'|'colorTemperatureKelvin';

@Component({selector:'app-settings-page',standalone:true,imports:[FormField,MatButtonModule,MatFormFieldModule,MatInputModule,MatRadioModule,ModePreviewComponent,RgbColorPickerComponent],templateUrl:'./settings-page.component.html',styleUrl:'./settings-page.component.scss',changeDetection:ChangeDetectionStrategy.OnPush})
export class SettingsPageComponent {
  private readonly api=inject(SettingsApiService);private readonly snackbar=inject(SnackbarService);private readonly destroyRef=inject(DestroyRef);
  protected readonly model=signal({dimmer:100,mode:'color_temperature' as 'rgb'|'color_temperature',rgbColor:'#88C0D0',colorTemperatureKelvin:3000});protected readonly pending=signal(false);protected readonly failure=signal<ApiFailure|null>(null);protected readonly controls=viewChildren<ElementRef<HTMLElement>>('formControl');private readonly colorPicker=viewChild(RgbColorPickerComponent);
  protected readonly settingsForm=form(this.model,p=>{required(p.dimmer,{message:'Dimmer is required'});validate(p.dimmer,({value})=>validateIntegerRange(value(),1,100));validate(p.rgbColor,({value})=>this.model().mode==='rgb'?validateUppercaseRgbHex(value()):undefined);validate(p.colorTemperatureKelvin,({value})=>this.model().mode==='color_temperature'?validateIntegerRange(value(),3000,6000):undefined);});
  constructor(){this.pending.set(true);this.api.getResetSettings().pipe(finalize(()=>this.pending.set(false)),takeUntilDestroyed(this.destroyRef)).subscribe({next:v=>{this.setSettings(v);this.snackbar.success('Settings loaded.');},error:(f:ApiFailure)=>{this.failure.set(f);this.snackbar.failure(f,'Could not load settings.');}});}
  protected updateRgbColor(value:string){this.model.update(current=>({...current,rgbColor:value}));this.settingsForm.rgbColor().markAsTouched();this.changed('rgbColor');}
  protected changed(field:SettingsField){this.failure.update(f=>f?{...f,fields:Object.fromEntries(Object.entries(f.fields).filter(([key])=>key!==field))}:null);}
  protected serverError(field:SettingsField){return this.failure()?.fields[field]??null;}protected clientError(field:SettingsField){return this.settingsForm[field]().errors()[0]?.message??null;}protected showClientError(field:SettingsField){const state=this.settingsForm[field]();return state.touched()&&state.invalid();}
  protected save(){if(this.pending())return;this.settingsForm().markAsTouched();if(this.settingsForm().invalid()){queueMicrotask(()=>this.focusFirstInvalid());return;}const v=this.model();const settings:ResetSettings=v.mode==='rgb'?{dimmer:v.dimmer,mode:'rgb',rgbColor:v.rgbColor.toUpperCase(),colorTemperatureKelvin:null}:{dimmer:v.dimmer,mode:'color_temperature',rgbColor:null,colorTemperatureKelvin:v.colorTemperatureKelvin};this.pending.set(true);this.failure.set(null);this.api.replaceResetSettings(settings).pipe(finalize(()=>this.pending.set(false)),takeUntilDestroyed(this.destroyRef)).subscribe({next:result=>{this.setSettings(result);this.snackbar.success('Settings saved.');},error:(failure:ApiFailure)=>{this.failure.set(failure);this.snackbar.failure(failure,'Could not save settings.');}});}
  private setSettings(v:ResetSettings){this.settingsForm().reset({dimmer:v.dimmer,mode:v.mode,rgbColor:v.rgbColor??'#88C0D0',colorTemperatureKelvin:v.colorTemperatureKelvin??3000});}
  private focusFirstInvalid(){const control=this.controls().find(item=>item.nativeElement.getAttribute('aria-invalid')==='true');if(control){control.nativeElement.focus();return;}if(this.model().mode==='rgb'&&this.settingsForm.rgbColor().invalid())this.colorPicker()?.focus();}
}
