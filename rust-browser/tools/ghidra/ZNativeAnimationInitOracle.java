// Check numeric phase/flags writes at the actual UNIT allocation boundary
// and occupied hardware initializers, leaving unrelated route masks external.
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeAnimationInitOracle extends ZNativeReactionsOracle{
 static final long H=0x59e6c2L;
 void invoke(EmulatorHelper e,long entry,long ecx,long edx,long...args)throws Exception{
  e.writeRegister("ESP",S);put(e,S,4,RET);for(int i=0;i<args.length;i++)put(e,S+4+i*4,4,args[i]);e.writeRegister("ECX",ecx);e.writeRegister("EDX",edx);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(entry),null,monitor))throw new Exception(e.getLastError());
  while(e.readRegister("EIP").longValue()!=RET){long pc=e.readRegister("EIP").longValue();if(pc!=0x438740&&pc!=0x41b950)throw new Exception("Unexpected "+Long.toHexString(pc));long stack=e.readRegister("ESP").longValue(),ret=read(e,stack,4);e.writeRegister("ESP",stack+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());}
 }
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  e.setBreakpoint(toAddr(0x438740));e.setBreakpoint(toAddr(0x41b950));e.setBreakpoint(toAddr(0x41b3ff));
  int[]colors={0,1,2,3,16,32,64,255};for(int team=0;team<4;team++)for(int color:colors){e.writeMemory(toAddr(U),new byte[192]);e.writeMemory(toAddr(P),new byte[256]);put(e,0x63238c,4,P);put(e,0x5849d3,1,0);put(e,0x584963,1,0);put(e,0x584962,1,0);put(e,0x57c952,1,team);put(e,0x622df9+team,1,color);e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,P+100);e.writeRegister("ECX",500);e.writeRegister("EDX",500);if(!e.run(toAddr(0x41b210),null,monitor))throw new Exception(e.getLastError());if(e.readRegister("EIP").longValue()==0x438740){long stack=e.readRegister("ESP").longValue(),ret=read(e,stack,4);e.writeRegister("ESP",stack+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());}if(e.readRegister("EIP").longValue()!=0x41b3ff)throw new Exception("Allocation prefix stop");out.println("{\"kind\":\"allocation\",\"team\":"+team+",\"color\":"+color+",\"phase\":"+read(e,U+0x35,1)+",\"flags\":"+read(e,U+0x64,1)+"}");}
  e.clearBreakpoint(toAddr(0x41b3ff));long[]entries={0x41bb70,0x458050,0x41bad0,0x41ba80,0x458090,0x41bb20,0x457ff0,0x4582c0,0x4582c0,0x4582c0,0x4582c0};int[]families={6,24,25,30,32,34,36,5,21,22,23};for(int k=0;k<entries.length;k++)for(int flags:new int[]{0,16,17,64,128,255}){e.writeMemory(toAddr(U),new byte[192]);e.writeMemory(toAddr(H),new byte[44]);put(e,U+8,4,H);put(e,U+0x30,1,families[k]);put(e,U+0x7c,1,16);put(e,U+0x64,1,flags);if(k>=7)invoke(e,entries[k],U,0);else if(k==1)invoke(e,entries[k],U,H);else invoke(e,entries[k],H,U,80);out.println("{\"kind\":\"hardware\",\"index\":"+k+",\"flagsInput\":"+flags+",\"phase\":"+read(e,U+0x35,1)+",\"flags\":"+read(e,U+0x64,1)+"}");}
 }finally{e.dispose();}}}
}
