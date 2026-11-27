// dil: tr
// A monthly task on the 31st keeps its day: completing it goes 31 Jan → 28 Feb → 31 Mar (not 28 Mar), via t.ayGun
const {esit}=RT;
S.todos=[{t:'Kira',due:'2099-01-31T10:00',tek:'ay'}];ciz();
const yol=[];for(let k=0;k<3;k++){isaretle(0);yol.push(S.todos[0].due.slice(0,10))}
esit('dates',yol,['2099-02-28','2099-03-31','2099-04-30']);
esit('own day kept',S.todos[0].ayGun,31);
