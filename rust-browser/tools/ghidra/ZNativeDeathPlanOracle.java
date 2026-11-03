// Executes the death emitter orchestration; removal/allocation/audio are explicit boundaries.
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeDeathPlanOracle extends ZNativeWorldEffectOracle{
 int eh,events;
 void event(int...vs){events++;for(int v:vs)eh=(eh^v)*16777619;}
 void execute(EmulatorHelper e,long fn)throws Exception{eh=0x811c9dc5;events=0;e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",O);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(fn),null,monitor))throw new Exception(e.getLastError());
 while(e.readRegister("EIP").longValue()!=RET){long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();
  if(pc==0x458ec0)event(0,(int)read(e,0x5849e7,1),(int)read(e,0x4a6e77,1));
  else if(pc==0x402160)event(1,(short)e.readRegister("ECX").intValue(),(short)read(e,0x622d80,2),(short)read(e,0x622d82,2),(int)read(e,0x4d18fb,1),(int)read(e,0x586a7c,2),(int)read(e,0x4d18f3,1),(int)read(e,0x586a78,4),(int)read(e,0x4d18e3,1),(int)read(e,0x4d18ef,1));
  else if(pc==0x46e660){event(2,e.readRegister("ECX").intValue());e.writeRegister("EAX",0);}else throw new Exception("Unexpected "+Long.toHexString(pc));
  long r=read(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());
 }}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{for(long a:new long[]{0x458ec0,0x402160,0x46e660})e.setBreakpoint(toAddr(a));
 int[]families={4,5,6,21,22,23,24,25,30,32,34,36,38};int[][]points={{160,100},{-49,-49},{-48,-48},{352,232},{353,233},{32767,-32768}};
 for(int fn=0;fn<2;fn++)for(int s=0;s<624;s++){e.writeMemory(toAddr(O),new byte[192]);int family=families[s%13],cl=s/13%4;int[]p=points[s/52%6];put(e,O+44,2,p[0]);put(e,O+46,2,p[1]);put(e,O+48,1,family);put(e,O+100,1,s&3);put(e,O+124,1,cl);put(e,0x622cb8,4,0);put(e,0x498c24,2,320);put(e,0x498c20,2,200);put(e,0x6323a0,1,(s>>1)&1);put(e,0x4d18ee,1,s&1);put(e,0x4a6f3b,1,s>>2&1);put(e,0x4a6f7e,1,s>>3&1);put(e,0x4a6528,4,O+0x1000);put(e,0x5849e7,1,0);put(e,0x4a6e77,1,0);execute(e,fn==0?0x41ef80:0x41e6e0);out.println("{\"scenario\":"+(fn*624+s)+",\"events\":"+events+",\"eventHash\":"+(eh&0xffffffffL)+",\"soundGate\":"+read(e,0x4d18ee,1)+",\"suppressed\":"+read(e,0x5849e7,1)+",\"removing\":"+read(e,0x4a6e77,1)+"}");}
 }finally{e.dispose();}}}
}
