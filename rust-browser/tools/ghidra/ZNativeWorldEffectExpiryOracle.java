// Executes403010/403090/403260 and direct expiry handlers402dd0/4012c0.
// Contact, particle allocation and audio boundaries are recorded in order.
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeWorldEffectExpiryOracle extends ZNativeWorldEffectOracle{
 int eventHash,eventCount;
 void event(int...values){eventCount++;for(int value:values)eventHash=(eventHash^value)*16777619;}
 @Override void run(EmulatorHelper e,long entry,long ecx,long edx,long...args)throws Exception{
  eventHash=0x811c9dc5;eventCount=0;dx=-999;dy=-999;sprite=-1;height=0;angle=0;transformed=0;e.writeRegister("ESP",S);put(e,S,4,RET);for(int i=0;i<args.length;i++)put(e,S+4+i*4,4,args[i]);e.writeRegister("ECX",ecx);e.writeRegister("EDX",edx);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(entry),null,monitor))throw new Exception(e.getLastError());
  while(e.readRegister("EIP").longValue()!=RET){long pc=e.readRegister("EIP").longValue(),stack=e.readRegister("ESP").longValue();
   if(pc==0x460910)e.writeRegister("EAX",(seed*7919^(calls++)*2027^0x7731a5c9)&0xffffffffL);
   else if(pc==0x20200+0x400000)event(2,(short)e.readRegister("EDX").intValue(),(short)read(e,stack+4,4),(short)read(e,stack+8,4),(int)read(e,0x4a6f42,1),(int)read(e,0x4a6e6f,1));
   else if(pc==0x41f2d0)event(3,(short)e.readRegister("ECX").intValue(),(short)e.readRegister("EDX").intValue(),(short)read(e,stack+4,4),(short)read(e,stack+8,4),(int)read(e,0x4a6f46,1));
   else if(pc==0x402160)event(4,(short)e.readRegister("ECX").intValue(),(short)read(e,0x622d80,2),(short)read(e,0x622d82,2),(int)read(e,0x4d18fb,1),(int)read(e,0x586a7c,2),(int)read(e,0x4d18f3,1),(int)read(e,0x586a78,4));
   else if(pc==0x46e660||pc==0x46e6d0){event(1,e.readRegister("ECX").intValue());e.writeRegister("EAX",0);}
   else if(pc==0x4666c0||pc==0x466ac0){dx=(short)e.readRegister("ECX").intValue();dy=(short)e.readRegister("EDX").intValue();sprite=(short)read(e,stack+4,4);if(pc==0x466ac0){height=(short)read(e,stack+8,4);angle=(int)read(e,stack+12,4);transformed=1;}}
   else if(pc==0x40bb50){}
   else throw new Exception("Unexpected "+Long.toHexString(pc));long ret=read(e,stack,4);e.writeRegister("ESP",stack+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());
  }
 }
 void initializeSpecial(EmulatorHelper e,int kind,int seed)throws Exception{
  this.seed=seed;calls=0;e.writeMemory(toAddr(O),new byte[44]);put(e,O+6,2,346);put(e,O+8,2,420);put(e,O+15,1,37);put(e,O+22,1,kind==1?60:kind==2?4:12);put(e,O+26,2,kind==0?5960:kind==1?1336:250);put(e,0x586a7c,2,kind==0?5960:kind==1?1336:250);run(e,kind==0?0x403090:kind==1?0x403010:0x403260,0,0,O);
 }
 @Override public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  for(long address:new long[]{0x460910,0x420200,0x41f2d0,0x402160,0x46e660,0x46e6d0})e.setBreakpoint(toAddr(address));
  for(int kind=0;kind<3;kind++)for(seed=0;seed<64;seed++){initializeSpecial(e,kind,seed);out.println("{\"kind\":\"specialInit\",\"scenario\":"+(kind*64+seed)+",\"calls\":"+calls+",\"hash\":"+hash(e.readMemory(toAddr(O),44))+"}");}
  int[]codes={0,7,9,42,38,40,39,29,127};int[][]points={{-40,512},{-33,512},{-32,512},{-1,512},{0,0},{16,16},{512,512},{2032,2032},{2048,2048},{32767,32767},{-32768,-32768},{320,180}};int[][]projection={{0,0},{160,100},{-160,-100},{32767,-32768}};
  for(int kind=0;kind<2;kind++)for(seed=0;seed<128;seed++){
   initializeSpecial(e,kind,seed);calls=0;int[]point=points[seed%points.length],proj=projection[seed/12%4];put(e,O,2,7);put(e,O+2,2,-13);put(e,O+4,2,(seed&1)!=0?-1:0);put(e,O+6,2,point[0]-proj[0]);put(e,O+8,2,point[1]-proj[1]);put(e,O+18,2,400-seed);put(e,0x4a6520,2,proj[0]);put(e,0x4a6530,2,proj[1]);put(e,0x622cb8,2,seed%3*12);put(e,0x622cba,2,seed%5*8);put(e,0x498c24,2,320);put(e,0x498c20,2,200);put(e,0x6323a0,1,1);put(e,0x4d18ee,1,seed&1);put(e,0x4a6e72,1,0);put(e,0x4a6f42,1,0);put(e,0x4a6e6f,1,0);e.writeMemory(toAddr(0x58aaaa),new byte[16384]);int cell=((((point[1]&65535)*8)&0xff80)+(point[0]>>4))&65535;if(cell<16384)put(e,0x58aaaa+cell,1,codes[seed%codes.length]);run(e,kind==0?0x4012c0:0x402dd0,O,0);
   out.println("{\"kind\":\"effectExpiry\",\"scenario\":"+(kind*128+seed)+",\"hash\":"+hash(e.readMemory(toAddr(O),44))+",\"calls\":"+calls+",\"events\":"+eventCount+",\"eventHash\":"+(eventHash&0xffffffffL)+",\"friendly\":"+read(e,0x4a6e72,1)+",\"suppressed\":"+read(e,0x4a6f42,1)+",\"quiet\":"+read(e,0x4a6e6f,1)+",\"soundGate\":"+read(e,0x4d18ee,1)+"}");
  }
 }finally{e.dispose();}}}
}
