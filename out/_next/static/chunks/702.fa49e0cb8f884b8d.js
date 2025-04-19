"use strict";(self.webpackChunk_N_E=self.webpackChunk_N_E||[]).push([[702],{4702:(e,t,s)=>{s.r(t),s.d(t,{default:()=>f});var l=s(5155),a=s(2115),i=s(3459),n=s(3286),c=s(6306),r=s(6517),o=s(7238),d=s(7712),x=s(4416),m=s(8164),h=s(3897),u=s(4611);let f=()=>{let[e,t]=(0,a.useState)(!1),s=(0,a.useRef)(null),[f,b]=(0,a.useState)(null),[j,p]=(0,a.useState)([]),[v,N]=(0,a.useState)(""),[g,w]=(0,a.useState)(0),[y,k]=(0,a.useState)(0),[S,C]=(0,a.useState)(""),z=(0,a.useRef)(!1),A=u.p8.getCurrent(),[_,T]=(0,a.useState)(!1),[F,I]=(0,a.useState)(!0);(0,a.useEffect)(()=>{(async()=>{await A.setAlwaysOnTop(!0),I(!0)})()},[]);let L=async()=>{let e=!F;I(e),await A.setAlwaysOnTop(e)},K=async()=>{await A.minimize()},E=async()=>{await A.close()},D=async()=>{try{z.current=!0,s.current=await i.I8.invoke("detect_arduino"),t(!0),await i.I8.invoke("start_streaming",{portName:s.current,stream_name:"UDL"})}catch(e){console.error("Failed to connect to device:",e)}},R=async()=>{try{await i.I8.invoke("start_wifistreaming"),z.current=!0,t(!0)}catch(e){console.error("Failed to connect to device:",e)}},M=async()=>{try{T(!0),z.current=!0,await i.I8.invoke("scan_ble_devices")}catch(e){console.error("Failed to connect to device:",e)}};return(0,a.useEffect)(()=>{let e=[];return(async()=>{let t=await (0,h.KT)("connection",e=>{N(e.payload)});e.push(t);let s=await (0,h.KT)("bleDevices",e=>{let t=e.payload;0===t.length&&N("No NPG device"),p(t)});e.push(s);let l=await (0,h.KT)("samplerate",e=>{w(Math.ceil(Number(e.payload)))});e.push(l);let a=await (0,h.KT)("samplelost",e=>{k(e.payload)});e.push(a);let i=await (0,h.KT)("lsl",e=>{C(e.payload)});e.push(i)})(),()=>{for(let t of e)t()}},[]),(0,l.jsx)(l.Fragment,{children:(0,l.jsxs)("div",{className:" flex-col bg-gray-200 overflow-hidden",children:[(0,l.jsx)("div",{className:"w-full",children:(0,l.jsxs)("div",{className:"flex justify-between items-center w-full h-12 px-4 bg-gray-800 text-white select-none","data-tauri-drag-region":!0,children:[(0,l.jsxs)("div",{className:"flex space-x-3",children:[(0,l.jsx)("button",{onClick:()=>{b("serial"),T(!1),p([]),D()},className:"transition-colors duration-300 hover:text-blue-400 ".concat("serial"===f?"text-green-500":""),title:"Serial",disabled:null!==f,children:(0,l.jsx)(n.A,{size:20})}),(0,l.jsx)("button",{onClick:()=>{b("bluetooth"),p([]),M()},className:"transition-colors duration-300 hover:text-blue-400 ".concat("bluetooth"===f?"text-green-500":""),title:"Bluetooth",disabled:null!==f,children:(0,l.jsx)(c.A,{size:20})}),(0,l.jsx)("button",{onClick:()=>{b("wifi"),T(!1),p([]),R()},className:"transition-colors duration-300 hover:text-blue-400 ".concat("wifi"===f?"text-green-500":""),title:"WiFi",disabled:null!==f,children:(0,l.jsx)(r.A,{size:20})})]}),(0,l.jsx)("div",{className:"flex items-center px-2 font-semibold text-sm tracking-wide text-shadow-sm select-none","data-tauri-drag-region":!0,children:"Chords LSL Connector"}),(0,l.jsxs)("div",{className:"flex space-x-3",children:[(0,l.jsx)("button",{onClick:L,className:"".concat(F?"text-green-400":"text-white"," hover:text-green-300"),title:"Toggle Always on Top",children:(0,l.jsx)(o.A,{size:20})}),(0,l.jsx)("button",{onClick:K,className:"hover:text-yellow-400",title:"Minimize",children:(0,l.jsx)(d.A,{size:20})}),(0,l.jsx)("button",{onClick:E,className:"hover:text-red-400",title:"Close",children:(0,l.jsx)(x.A,{size:20})})]})]})}),(0,l.jsxs)("div",{className:"flex  relative  ",children:[(0,l.jsx)("div",{className:"w-full md:w-1/2 flex flex-col",children:(0,l.jsxs)("div",{className:"flex-1  grid place-items-center bg-slate-50 ",children:[!_&&(0,l.jsx)(l.Fragment,{children:f?({serial:(0,l.jsx)(m.A,{size:50,className:"transition-colors duration-300 ".concat(e&&"serial"===f?"text-green-500":"text-gray-500")}),bluetooth:(0,l.jsx)(c.A,{size:50,className:"transition-colors duration-300 ".concat(e&&"bluetooth"===f?"text-green-500":"text-gray-500")}),wifi:(0,l.jsx)(r.A,{size:50,className:"transition-colors duration-300 ".concat(e&&"wifi"===f?"text-green-500":"text-gray-500")})})[f]:(0,l.jsxs)("div",{className:"text-gray-400 text-sm text-center",children:[(0,l.jsx)("p",{children:"Select a"}),(0,l.jsx)("p",{children:"connection"})]})}),_&&!e&&(0,l.jsx)("div",{className:"max-w-md relative  rounded overflow-hidden",children:(0,l.jsx)("div",{className:"max-h-[60vh] overflow-y-auto",children:j.length>0?(0,l.jsxs)("ul",{className:"",children:[" ",j.map(e=>(0,l.jsxs)("li",{className:"flex items-center pl-1  hover:bg-gray-100",children:[(0,l.jsx)("label",{htmlFor:"device-".concat(e.id),className:"flex-1 text-gray-700 cursor-pointer",children:e.name||"Unknown Device (".concat(e.id,")")}),(0,l.jsx)("button",{onClick:async()=>{N(await i.I8.invoke("connect_to_ble",{deviceId:e.id})),t(!0),T(!1)},className:"ml-2 bg-blue-500 hover:bg-blue-600 text-black mr-3 rounded text-sm transition-colors px-1 py-1",children:(0,l.jsx)(m.A,{size:14})})]},e.id))]}):(0,l.jsx)("p",{className:" max-h-[60vh] text-black ",children:"Scanning for devices..."})})})]})}),(0,l.jsx)("div",{className:"absolute left-1/2 top-0 transform -translate-x-1/2 w-px h-full bg-slate-200 z-10 md:static md:w-px md:h-full"}),(0,l.jsx)("div",{className:"w-full md:w-1/2 flex flex-col",children:(0,l.jsxs)("div",{className:"flex flex-col h-full",children:[(0,l.jsx)("div",{className:"flex flex-col justify-center pl-2 border-x border-b  border-slate-200 bg-slate-50 shadow-sm min-h-[20px]  transition-all",children:(0,l.jsxs)("p",{className:"text-black ",children:[(0,l.jsx)("span",{className:"text-lg  font-semibold",children:"Status: "}),v||"Not Connected"]})}),(0,l.jsx)("div",{className:" flex flex-col justify-center pl-2  border border-slate-200 bg-slate-50 shadow-sm min-h-[20px]   transition-all",children:(0,l.jsxs)("p",{className:"text-black ",children:[(0,l.jsx)("span",{className:"text-lg  font-semibold",children:"Sampling Rate:  "}),g||"No "]})}),(0,l.jsx)("div",{className:" flex flex-col justify-center pl-2  border border-slate-200 bg-slate-50 shadow-sm min-h-[20px]  transition-all",children:(0,l.jsxs)("p",{className:"text-black ",children:[(0,l.jsx)("span",{className:"text-lg  font-semibold",children:"Sample Lost: "}),y]})}),(0,l.jsx)("div",{className:" flex flex-col justify-center  pl-2 border-x border-t border-slate-200 bg-slate-50 shadow-sm min-h-[20px]  transition-all",children:(0,l.jsxs)("p",{className:"text-black ",children:[(0,l.jsx)("span",{className:"text-lg  font-semibold",children:"LSL: "}),S||"No lsl yet"]})})]})})]})]})})}}}]);