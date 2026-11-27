// dil: en
// A count of 1 takes the singular (Ln's English singular), more takes the plural; Turkish has one form
const {esit}=RT;
esit('English',[1,2].map(n=>n+Ln(' görev',' tasks',n,' task')),['1 task','2 tasks']);
esit('no singular given: the plural stays',Ln(' görev',' rounds',1),' rounds');
