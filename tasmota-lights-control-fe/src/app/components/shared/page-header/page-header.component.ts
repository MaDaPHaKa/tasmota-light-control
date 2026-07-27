import { Component, input } from '@angular/core';

@Component({
  selector: 'app-page-header',
  imports: [],
  templateUrl: './page-header.component.html',
  styleUrl: './page-header.component.scss',
})
export class PageHeader {
  readonly sectionEyebrow = input.required<string>();
  readonly sectionName = input.required<string>();
  readonly sectionDescription = input<string>();
}
