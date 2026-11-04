import ghidra.app.script.GhidraScript;import ghidra.app.emulator.EmulatorHelper;import java.io.*;
// Private executable writer/clock proof. No loaded-program/DB mutation.
public class ZRound38WreckWritersOracle extends GhidraScript{
 static final long O=0x612f80L,U=0x5e30c8L,S=0x7fff0000L,RET=0x12345000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}void run(EmulatorHelper e,long p)throws Exception{if(!e.run(toAddr(p),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int s=0;s<16;s++){EmulatorHelper e=new EmulatorHelper(currentProgram);try{int branch=s%2;put(e,O+26,1,0xa5);put(e,O+38,1,0x5a);for(int i=0;i<5;i++)put(e,0x622cdc+i*4,4,s*7919+i+1);put(e,0x622dc4,1,s/8);e.writeRegister("EDI",O);e.writeRegister("ESI",U);e.writeRegister("ESP",S);e.setBreakpoint(toAddr(branch==0?0x458def:0x458e16));run(e,branch==0?0x458dd4:0x458dff);out.println("{\"suite\":\"58af0writer\",\"scenario\":"+s+",\"branch\":"+branch+",\"mp\":"+(s/8)+",\"flags1a\":"+read(e,O+26,1)+",\"variant26\":"+read(e,O+38,1)+",\"lifetime24\":"+read(e,O+36,2)+",\"flags19\":"+read(e,O+19,1)+",\"rng0\":"+read(e,0x622cdc,4)+"}");}finally{e.dispose();}}
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{put(e,0x622dd6,2,0);put(e,0x622d40,4,0);put(e,0x622c88,4,0);put(e,0x622dc0,1,0);e.setBreakpoint(toAddr(RET));for(int t=0;t<18;t++){e.writeRegister("ESP",S);put(e,S,4,RET);run(e,0x434a70);out.println("{\"suite\":\"434a70clock\",\"callback\":"+(t+1)+",\"six622dd6\":"+read(e,0x622dd6,1)+",\"eight622dd7\":"+read(e,0x622dd7,1)+",\"four622d40\":"+read(e,0x622d40,2)+",\"mode622dc0\":"+read(e,0x622dc0,1)+"}");}}finally{e.dispose();}
 }}
}
