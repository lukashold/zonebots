//58af0 conversion with the terrain-placement and first-free allocation boundaries recorded.
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeWreckOracle extends ZNativeWorldEffectOracle{
 int eh,events,allocation;static final long H=O+0x400,P=O+0x600;
 void event(int...vs){events++;for(int v:vs)eh=(eh^v)*16777619;}
 void execute(EmulatorHelper e)throws Exception{eh=0x811c9dc5;events=0;e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",O);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(0x458af0),null,monitor))throw new Exception(e.getLastError());
 while(e.readRegister("EIP").longValue()!=RET){long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();
  if(pc==0x41ddb0)event(0);
  else if(pc==0x458a40){int x=(short)e.readRegister("EDX").intValue(),y=(short)read(e,sp+4,4),kind=(short)read(e,sp+8,4);event(1,x,y,kind);e.writeRegister("EAX",allocation);if(allocation==0){put(e,read(e,sp+12,4),4,P);put(e,P,2,x);put(e,P+2,2,y);put(e,P+4,4,0x498f38+kind*26);put(e,P+8,4,-1);put(e,P+12,1,((y>>3)&240)+(x>>7));put(e,P+13,1,((read(e,O+100,1)&3)|24)*2);put(e,P+18,1,0);put(e,P+36,2,0);}}
  else if(pc==0x420890)event(2,(short)e.readRegister("EDX").intValue(),(short)read(e,sp+4,4));
  else if(pc==0x44de00)event(3);
  else if(pc==0x460910)e.writeRegister("EAX",(seed*7919^(calls++)*2027^0x7731a5c9)&0xffffffffL);
  else throw new Exception("Unexpected "+Long.toHexString(pc));
  long r=read(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());
 }}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{for(long a:new long[]{0x41ddb0,0x458a40,0x420890,0x44de00,0x460910})e.setBreakpoint(toAddr(a));int[]families={4,5,6,21,22,23,24,25,30,32,34,36,38};int[]hps={-300,-1,0,1,24,25,125};
 for(seed=0;seed<1664;seed++){calls=0;allocation=seed>>5&1;e.writeMemory(toAddr(O),new byte[192]);byte[]hardware=new byte[64],object=new byte[44];for(int i=0;i<64;i++)hardware[i]=(byte)(i*11+seed);for(int i=0;i<44;i++)object[i]=(byte)(i*17+seed);e.writeMemory(toAddr(H),hardware);e.writeMemory(toAddr(P),object);put(e,H+8,2,hps[seed/13%7]);put(e,H+10,2,300);put(e,O+8,4,H);put(e,O+44,2,346);put(e,O+46,2,420);put(e,O+48,1,families[seed%13]);put(e,O+100,1,seed&3);put(e,O+124,1,seed/91%4);put(e,0x4a6f7e,1,seed>>1&1);put(e,0x5849e7,1,seed>>2&1);put(e,0x4a6e77,1,seed>>3&1);put(e,0x4d02f6,1,seed>>4&1);execute(e);out.println("{\"scenario\":"+seed+",\"events\":"+events+",\"eventHash\":"+(eh&0xffffffffL)+",\"calls\":"+calls+",\"objectHash\":"+hash(e.readMemory(toAddr(P),44))+",\"family\":"+read(e,O+48,1)+",\"hardwareFreed\":"+(read(e,H,2)==65535?1:0)+"}");}
 }finally{e.dispose();}}}
}
