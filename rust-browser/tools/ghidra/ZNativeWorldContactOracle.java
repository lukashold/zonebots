// Numeric native world-contact evidence. Graphics/audio and destructive
// teardown callbacks are intercepted after their decision/HP effects.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeWorldContactOracle extends GhidraScript {
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,O=0x612f80L,B=0x5c5e38L,R=0x6151e0L,M=0x1100000L,P=M+0x1000,OUT=M+0x2000,W=M+0x3000;
 String mode;boolean destroyed,firstDead,secondDead,nearby,order;int sampleX,sampleY,destX,destY,randomCalls;int[]random=new int[2];
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void run(EmulatorHelper e,long address,long ecx,long edx,long...args)throws Exception{
  destroyed=false;firstDead=false;secondDead=false;nearby=false;order=false;destX=-999;destY=-999;randomCalls=0;
  e.writeRegister("ESP",S);put(e,S,4,RET);for(int i=0;i<args.length;i++)put(e,S+4+i*4,4,args[i]);e.writeRegister("ECX",ecx);e.writeRegister("EDX",edx);e.setBreakpoint(toAddr(RET));
  if(!e.run(toAddr(address),null,monitor))throw new Exception(e.getLastError());
  while(e.readRegister("EIP").longValue()!=RET){long pc=e.readRegister("EIP").longValue(),stack=e.readRegister("ESP").longValue();
   if(pc==0x41c600){put(e,read(e,stack+4,4),4,O);put(e,read(e,stack+8,4),4,0x498f38);e.writeRegister("EAX",1);}
   else if(pc==0x41c290){nearby=true;sampleX=(short)read(e,e.readRegister("ECX").longValue(),2);sampleY=(short)read(e,e.readRegister("EDX").longValue(),2);put(e,read(e,stack+4,4),4,B);e.writeRegister("EAX",mode.equals("fort")?0:1);}
   else if(pc==0x40e790){destroyed=true;}
   else if(pc==0x4228f0){firstDead=true;}
   else if(pc==0x422850){secondDead=true;}
   else if(pc==0x4093e0){put(e,read(e,stack+4,4),4,R);e.writeRegister("EAX",1);}
   else if(pc==0x4094e0)e.writeRegister("EAX",0);
   else if(pc==0x44cd60){put(e,0x62d3a8,1,0);put(e,0x62d3a9,1,255);}
   else if(pc==0x460910)e.writeRegister("EAX",random[randomCalls++]);
   else if(pc==0x4525a0){order=true;destX=(short)e.readRegister("EDX").intValue();destY=(short)read(e,stack+4,4);e.writeRegister("EAX",0);}
   else if(pc!=0x402160&&pc!=0x41e990&&pc!=0x41ea90&&pc!=0x46e960&&pc!=0x46f2d0&&pc!=0x409a60&&pc!=0x409870&&pc!=0x41f870&&pc!=0x459d10&&pc!=0x43f650&&pc!=0x409690)throw new Exception("Unexpected "+mode+" stop "+Long.toHexString(pc));
   long ret=read(e,stack,4);e.writeRegister("ESP",stack+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());
  }
 }
 void bp(EmulatorHelper e,long...addresses){for(long a:addresses)e.setBreakpoint(toAddr(a));}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  EmulatorHelper e=new EmulatorHelper(currentProgram);mode="objects";
  try{for(int scenario=0;scenario<4096;scenario++){
   e.writeMemory(toAddr(O),new byte[44*200]);for(int i=0;i<200;i++)put(e,O+i*44,2,65535);
   for(int i=0;i<3;i++){long o=O+i*44,m=M+i*26;put(e,o,2,((scenario>>(i+6))&1)!=0?65535:500+i*6);put(e,o+2,2,500+i*6);put(e,o+4,4,m);put(e,o+13,1,scenario*37+i*83);put(e,m+18,1,scenario*13+i*17);put(e,m+20,1,scenario*43+i*7);}
   put(e,0x4a6e6a,1,scenario&1);put(e,0x4a6e6b,1,(scenario>>1)&1);run(e,0x41c600,500+scenario%49-24,500+scenario/49%49-24,OUT,OUT+4);
   int slot=(e.readRegister("EAX").intValue()&255)==0?-1:(int)((read(e,OUT,4)-O)/44);
   out.println("{\"kind\":\"object\",\"scenario\":"+scenario+",\"slot\":"+slot+",\"extentFlag\":"+read(e,0x4a6e6b,1)+"}");
  }}finally{e.dispose();}
  e=new EmulatorHelper(currentProgram);mode="buildings";
  try{for(int scenario=0;scenario<4096;scenario++){
   e.writeMemory(toAddr(B),new byte[88*35]);for(int i=0;i<35;i++)put(e,B+i*88+4,1,255);
   int[]hp={-1,0,1,500};for(int i=0;i<3;i++){long b=B+i*88;put(e,b,2,500+i*24);put(e,b+2,2,500+i*24);put(e,b+4,1,(scenario+i)%10-1);put(e,b+30,2,hp[(scenario>>(i+2))&3]);}
   put(e,P,2,500+scenario%193-16);put(e,P+2,2,500+scenario/193%193-16);put(e,0x4a69f7,1,scenario&1);run(e,(scenario&2)!=0?0x41c4f0:0x41c290,P,P+2,OUT);
   int slot=(e.readRegister("EAX").intValue()&255)==0?-1:(int)((read(e,OUT,4)-B)/88);
   out.println("{\"kind\":\"building\",\"scenario\":"+scenario+",\"slot\":"+slot+",\"x\":"+(short)read(e,P,2)+",\"y\":"+(short)read(e,P+2,2)+"}");
  }}finally{e.dispose();}
  e=new EmulatorHelper(currentProgram);mode="objectDamage";
  try{bp(e,0x41c600,0x402160,0x41e990,0x41ea90);int[]strengths={-32768,-1,0,1,10,127,255,32767};int scenario=0;
   for(int hp=0;hp<256;hp++)for(int strength:strengths)for(int force=0;force<2;force++){
    e.writeMemory(toAddr(O),new byte[44]);e.writeMemory(toAddr(0x498f38),new byte[26]);put(e,O,2,500);put(e,O+2,2,500);put(e,O+4,4,0x498f38);put(e,O+15,1,hp);put(e,0x4a6f3a,1,0);put(e,0x4a6f47,1,force);
    run(e,0x41eb90,1000,1000,strength);out.println("{\"kind\":\"objectDamage\",\"scenario\":"+scenario+++",\"hp\":"+read(e,O+15,1)+",\"dead\":"+(read(e,O,2)==65535?1:0)+"}");
   }
  }finally{e.dispose();}
  e=new EmulatorHelper(currentProgram);mode="buildingDamage";
  try{bp(e,0x41c290,0x40e790,0x46e960);int scenario=0;
   for(int hp:new int[]{-32768,-1,0,1,500,32767})for(int strength:new int[]{-32768,-1,0,1,10,127,255,32767})for(int fire=0;fire<2;fire++){
    e.writeMemory(toAddr(B),new byte[88]);put(e,B+30,2,hp);put(e,0x4a6e73,1,fire);put(e,0x4a6e72,1,0);run(e,0x40e920,P,500,600,strength);
    out.println("{\"kind\":\"buildingDamage\",\"scenario\":"+scenario+++",\"hp\":"+(short)read(e,B+30,2)+",\"dead\":"+(destroyed?1:0)+",\"x\":"+sampleX+",\"y\":"+sampleY+"}");
   }
  }finally{e.dispose();}
  e=new EmulatorHelper(currentProgram);mode="fort";
  try{bp(e,0x41c290,0x4228f0,0x422850);int scenario=0;int[][]points={{488,460},{488,620},{648,460},{648,620},{487,460},{649,620},{688,676},{848,836},{687,676},{849,836},{500,500},{700,700},{588,560},{788,736},{32767,32767},{-32768,-32768}};
   for(int source=-1;source<3;source++)for(int allow=0;allow<2;allow++)for(int[]origin:points)for(int hp:new int[]{0,1,500,32767})for(int strength:new int[]{0,1,127,32767}){
    put(e,0x622d54,2,500);put(e,0x622d56,2,500);put(e,0x622d58,2,700);put(e,0x622d5a,2,700);put(e,0x622d64,2,hp);put(e,0x622d68,2,hp);put(e,P+0x22,1,source);put(e,0x4a6e72,1,allow);
    run(e,0x41fe30,source==-1?0:P,origin[0],origin[1],strength);
    out.println("{\"kind\":\"fort\",\"scenario\":"+scenario+++",\"hp0\":"+(short)read(e,0x622d64,2)+",\"hp1\":"+(short)read(e,0x622d68,2)+",\"dead0\":"+(firstDead?1:0)+",\"dead1\":"+(secondDead?1:0)+",\"near\":"+(nearby?1:0)+"}");
   }
  }finally{e.dispose();}
  e=new EmulatorHelper(currentProgram);mode="bridge";
  try{bp(e,0x4093e0,0x46f2d0,0x409a60,0x409870,0x41f870,0x459d10,0x43f650,0x4094e0,0x409690);int scenario=0;
   for(int hp:new int[]{-32768,-1,0,1,100,32767})for(int strength:new int[]{-32768,-1,0,1,10,127,255,32767})for(int flags:new int[]{0,2,32,255}){
    e.writeMemory(toAddr(R),new byte[32]);put(e,R+8,2,hp);put(e,R+10,2,200);put(e,R+15,1,flags);put(e,0x4a6e72,1,0);run(e,0x409c50,P,strength,500);
    out.println("{\"kind\":\"bridgeDamage\",\"scenario\":"+scenario+++",\"hp\":"+(short)read(e,R+8,2)+",\"flags\":"+read(e,R+15,1)+",\"dead\":"+(e.readRegister("EAX").intValue()&255)+"}");
   }
  }finally{e.dispose();}
  e=new EmulatorHelper(currentProgram);mode="dodge";
  try{bp(e,0x44cd60,0x460910,0x4525a0);int[]classes={0,1,2,3,15,16,17,18},states={0,1,2,7,18,43,60,255};
   for(int scenario=0;scenario<768;scenario++){
    e.writeMemory(toAddr(U),new byte[0xc0*2]);e.writeMemory(toAddr(P),new byte[0x40]);e.writeMemory(toAddr(W),new byte[44]);int skill,x,y,tx,ty,cls,state,flags31,flags64,flags69;
    if(scenario<512){cls=classes[scenario&7];state=states[scenario>>3&7];flags31=4|((scenario>>6)&1)*64;flags64=((scenario>>7)&1)*64;flags69=((scenario>>8)&1)*128;skill=120;x=500;y=500;tx=508;ty=508;}
    else{int q=scenario-512;cls=0;state=1;flags31=4;flags64=0;flags69=(q&1)*128;skill=q*43;x=500+q*13%83-41;y=500+q*17%83-41;tx=508+(q%5-2)*32;ty=508+(q/5%5-2)*32;}
    put(e,U+0x2c,2,500);put(e,U+0x2e,2,500);put(e,U+0x31,1,flags31);put(e,U+0x64,1,flags64);put(e,U+0x69,1,flags69);put(e,U+0x7c,1,cls);put(e,U+0x35,1,state);put(e,U+2,1,skill);put(e,U+8,4,W);put(e,U+0xc0+0x31,1,5);
    put(e,P,2,x);put(e,P+2,2,y);put(e,P+0x24,1,1);put(e,P+0x26,2,tx);put(e,P+0x28,2,ty);random[0]=scenario*2027^0x7731a5c9;random[1]=scenario*7919^0x12348765;
    run(e,0x462fd0,P,0);out.println("{\"kind\":\"dodge\",\"scenario\":"+scenario+",\"order\":"+(order?1:0)+",\"x\":"+destX+",\"y\":"+destY+",\"flags64\":"+read(e,U+0x64,1)+",\"counter\":"+read(e,U+0x8c,1)+",\"hardware\":"+read(e,W+0x17,1)+",\"calls\":"+randomCalls+"}");
   }
  }finally{e.dispose();}
 }}
}
