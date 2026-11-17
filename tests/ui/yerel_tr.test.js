// dil: tr
// Turkish defaults stay as before: 24-hour clock, Monday-first weeks, °C, Turkish holidays
const {esit}=RT;
esit('tr defaults',[OTO12,saat(new Date(2026,10,4,18,5)),OTO_HB,GUN()[0],FAH(),tatilId()],
  [false,'18:05',1,'Pzt',false,'tr.turkish#holiday@group.v.calendar.google.com']);
S.ayar.saatBicim='h12';esit('12 h in Turkish',saat(new Date(2026,10,4,18,5)),'6:05 ÖS');
