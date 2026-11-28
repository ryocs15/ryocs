// dil: tr
// Mail account chips sit inside the scrolling mail list: the selected chip's frame keeps room from the list's edge (it was cut)
const {dogru,bek}=RT;const f0=window.fetch;
const v={okunmamis:3,liste:[{id:'a',hesap:'a@x.com',konu:'Merhaba',kimden:'x@y',t:Date.now(),yeni:true}],hesaplar:[{eposta:'a@x.com',ana:true,okunmamis:2},{eposta:'b@x.com',okunmamis:1}]};
window.fetch=async(p,a)=>p.startsWith('/api/mail?')?{json:async()=>({durum:'ok',veri:v})}:f0(p,a);
for(const x of document.querySelectorAll('dialog[open]'))x.close();kartAc('mail',true);await bek(100);await mail(true);await bek(100);
const l=$('#mail').getBoundingClientRect(),c=$('#mail .mailHs [aria-pressed=true]').getBoundingClientRect();
dogru(`selected chip has room on top and left (${Math.round(c.top-l.top)}, ${Math.round(c.left-l.left)} px)`,c.top-l.top>=4&&c.left-l.left>=4);
