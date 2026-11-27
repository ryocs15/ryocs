// dil: tr
// Gmail card setting: how many emails per account are fetched (5–50, 15 by default); a change refetches with the new count
const {esit,dogru,q,bek}=RT;
const istek=[];const f0=window.fetch;
window.fetch=async(p,a)=>{if(String(p).startsWith('/api/mail?')){istek.push(p);return{json:async()=>({durum:'ok',veri:{okunmamis:0,liste:[],hesaplar:[]}})}}return f0(p,a)};
kartAc('mail',true);await mail(true);
esit('default 15',istek.at(-1),'/api/mail?n=15');
kartAyar('mail');const x=document.querySelector('dialog[open] input[data-a=mailSayi]');
dogru('setting shown',!!x);esit('shows the default',x.value,'15');
x.value='30';x.dispatchEvent(new Event('change',{bubbles:true}));await bek(50);
esit('saved',S.ayar.mailSayi,30);esit('refetched with the new count',istek.at(-1),'/api/mail?n=30');
x.value='500';x.dispatchEvent(new Event('change',{bubbles:true}));await bek(50);
esit('clamped to 50',[x.value,S.ayar.mailSayi],['50',50]);
