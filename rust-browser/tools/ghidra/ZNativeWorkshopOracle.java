// Native BUILD0 callback/admission and the shared150-slot spark constructor.
// Only graphics, fragment damage, command dispatch and audio leaves are hooks.
import ghidra.app.emulator.EmulatorHelper;import java.io.*;import java.util.*;
public class ZNativeWorkshopOracle extends ZNativeWorldEffectOracle{
 static final long B=0x5c5e38,U=0x5e30c8,W=0x1900000,C=0x1910000,G=0x60ec00,CR=0x12346000;
 int scenario,rng,crt,stamps,sparks,dispatch,complete,fragments;long event;
 long mix(long h,long v){return((h^(v&0xffffffffL))*16777619)&0xffffffffL;}
 void invoke(EmulatorHelper e,long entry,boolean callback)throws Exception{
  rng=crt=stamps=sparks=dispatch=complete=fragments=0;event=2166136261L;e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,B);e.writeRegister("ECX",entry==0x41dbf0?346:U);e.writeRegister("EDX",entry==0x41dbf0?420:0);
  for(long a:new long[]{RET,CR,0x460910,0x4666c0,0x40bb50,0x41f610,0x40ac30,0x467b80,0x453ff0,0x452500,0x438740,0x45ef00,0x45a9b0,0x46f690,0x46e660,0x4396a0})e.setBreakpoint(toAddr(a));
  if(callback){e.setBreakpoint(toAddr(0x40cc90));e.setBreakpoint(toAddr(0x41dbf0));}
  else{e.clearBreakpoint(toAddr(0x40cc90));e.clearBreakpoint(toAddr(0x41dbf0));}
  if(!e.run(toAddr(entry),null,monitor))throw new Exception(e.getLastError());
  while(e.readRegister("EIP").longValue()!=RET){long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue(),result=0;
   if(pc==0x460910){result=(scenario*7919^(rng++)*2027^0x7731a5c9)&0xffffffffL;}
   else if(pc==CR){result=(scenario*3571+(crt++)*131+0x1743)&32767;}
   else if(pc==0x40cc90){stamps++;}
   else if(pc==0x41dbf0){sparks++;}
   else if(pc==0x41f610){fragments++;}
   else if(pc==0x45ef00){dispatch++;}
   else if(pc==0x46f690){complete++;}
   else if(pc==0x46e660){result=0;}
   else if(pc!=0x4666c0&&pc!=0x40bb50&&pc!=0x40ac30&&pc!=0x467b80&&pc!=0x453ff0&&pc!=0x452500&&pc!=0x438740&&pc!=0x45a9b0&&pc!=0x4396a0)throw new Exception("Unexpected "+Long.toHexString(pc));
   if(pc!=0x4666c0&&pc!=0x40bb50){event=mix(event,pc);}
   e.writeRegister("EAX",result);long ret=read(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());
  }
 }
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  put(e,0x4740ac,4,CR);for(int mode=0;mode<3;mode++){int count=mode==0?4096:mode==1?1024:512;for(scenario=0;scenario<count;scenario++){int s=scenario;e.writeMemory(toAddr(B),new byte[35*88]);e.writeMemory(toAddr(U),new byte[192]);e.writeMemory(toAddr(W),new byte[44]);e.writeMemory(toAddr(C),new byte[32]);e.writeMemory(toAddr(G),new byte[72]);put(e,B,2,400);put(e,B+2,2,500);put(e,B+4,1,0);put(e,B+5,1,s*13);put(e,B+8,2,0);put(e,B+10,2,s*117);put(e,B+17,1,s*17);for(int j=18;j<22;j++)put(e,B+j,1,s*31+j*7);put(e,B+28,2,new int[]{1,5,100,1000,32767,-1,-32768}[s/7%7]);put(e,B+30,2,new int[]{0,1,40,100,1000,-1,32767,-32768}[s/49%8]);put(e,B+32,4,C);put(e,B+36,4,new int[]{0,1,25,26,27,884,100000,-1}[s/3%8]);put(e,B+58,1,32);put(e,B+62,1,new int[]{0,15,16,17,126,127,128,255}[s/11%8]);put(e,B+69,1,new int[]{0,1,2,4,5,128,255}[s/37%7]);put(e,C+8,1,new int[]{0,1,3,4,7,-1}[s%6]);int owner=(byte)read(e,C+8,1);put(e,0x4a68a4+(owner&3),1,new int[]{0,1,26,51,127,-1,-128}[s/5%7]);put(e,U+8,4,W);put(e,U+16,4,((500&0xfff0)*8+(400>>4)+0x181)+(mode==1&&s%11==0?1:0));put(e,U+44,2,424);put(e,U+46,2,548);put(e,U+49,1,new int[]{0,4,5,7}[s/13%4]);put(e,U+100,1,s*19);put(e,U+105,1,96);put(e,U+2,1,11);put(e,U+3,1,71);put(e,U+92,1,7);put(e,U+94,1,s%3==0?255:0);put(e,U+95,1,0);put(e,U+124,1,1);put(e,W+7,1,2);put(e,W+12,1,33);put(e,W+8,2,new int[]{0,1,99,100,32767,-1,-32768}[s/17%7]);put(e,W+10,2,100);put(e,0x632194,4,G);put(e,0x622dba,2,new int[]{1,12,17,-1}[s/23%4]);put(e,0x622d1a,2,s%2);put(e,0x622dc2,1,0);put(e,0x622e18,1,255);put(e,0x6323a0,1,s%2);put(e,0x681580,4,0);put(e,0x622dc0,1,s%2);put(e,0x622cb8,2,s%3==0?1000:0);put(e,0x622cba,2,s%3==0?1000:0);put(e,0x498c24,2,800);put(e,0x498c20,2,600);
   if(mode==0){invoke(e,0x40d050,true);out.println("{\"mode\":0,\"scenario\":"+s+",\"build\":"+hash(e.readMemory(toAddr(B),88))+",\"hp\":"+(short)read(e,W+8,2)+",\"flags69\":"+read(e,U+105,1)+",\"order\":"+read(e,U+92,1)+",\"flags9c\":"+read(e,U+156,1)+",\"protection\":"+read(e,U+170,1)+",\"groupFlags\":"+read(e,G+9,1)+",\"rng\":"+rng+",\"stamps\":"+stamps+",\"sparks\":"+sparks+",\"dispatch\":"+dispatch+",\"complete\":"+complete+",\"fragments\":"+fragments+"}");}
   else if(mode==1){invoke(e,0x44ebf0,false);out.println("{\"mode\":1,\"scenario\":"+s+",\"build\":"+hash(e.readMemory(toAddr(B),88))+",\"unit\":"+hash(e.readMemory(toAddr(U),192))+",\"hardware\":"+hash(e.readMemory(toAddr(W),44))+",\"group\":"+hash(e.readMemory(toAddr(G),72))+",\"dispatch\":"+dispatch+",\"selected\":"+read(e,0x622d1a,2)+"}");}
   else{int free=new int[]{0,1,149,150}[s%4];for(int j=0;j<150;j++){byte[]raw=new byte[48];for(int k=0;k<48;k++)raw[k]=(byte)((j*48+k)*17+s);e.writeMemory(toAddr(0x60cfe0+j*48),raw);put(e,0x60cfe0+j*48,2,j<free?100+j:-1);}if(s%8==0){put(e,0x60cfe0+48,2,346);put(e,0x60cfe0+50,2,444);put(e,0x60cfe0+52,2,346);put(e,0x60cfe0+54,2,444);}put(e,0x622dc4,1,s%3==0?1:0);invoke(e,0x41dbf0,false);out.println("{\"mode\":2,\"scenario\":"+s+",\"pool\":"+hash(e.readMemory(toAddr(0x60cfe0),150*48))+",\"crt\":"+crt+"}");}
  }}}finally{e.dispose();}}}
}
