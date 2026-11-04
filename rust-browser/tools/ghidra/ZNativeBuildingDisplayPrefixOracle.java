//40ad60's actual35-record loop; per-BUILD callbacks, work/rate helpers and
//UI/raster routines are boundaries. Captures word36 at the callback, not
//after a later producer update. The two earlier bulk helpers are boundaries.
import ghidra.app.script.GhidraScript;import ghidra.app.emulator.EmulatorHelper;import java.io.PrintWriter;
public class ZNativeBuildingDisplayPrefixOracle extends GhidraScript{
 static final long B=0x5c5e38L,CP=0x5c3f0cL,S=0x7fff0000L,RET=0x12345000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  long callback=currentProgram.getMemory().getInt(toAddr(0x498cb4+8*0x4c))&0xffffffffL;for(long a:new long[]{RET,0x45ccb0,0x40a790,0x40a9c0,callback})e.setBreakpoint(toAddr(a));put(e,0x4d02db,1,0);put(e,0x4d02d3,1,0);put(e,0x57c94b,1,0);put(e,CP+8,1,0);
  for(int scenario=0;scenario<1024;scenario++){int value=scenario<10?new int[]{0,1,2,68,32767,-32768,-1,-2,-32767,65535}[scenario]:scenario*997;int slot=scenario%35;long record=B+slot*88;e.writeMemory(toAddr(B),new byte[35*88]);for(int k=0;k<35;k++)put(e,B+k*88+4,1,255);put(e,record+4,1,8);put(e,record+30,2,1);put(e,record+32,4,CP);put(e,record+54,2,value);put(e,record+76,4,-1);put(e,S,4,RET);e.writeRegister("ESP",S);int calls=0,observed=-999999;
   if(!e.run(toAddr(0x40ad60),null,monitor))throw new Exception(e.getLastError());for(int budget=0;e.readRegister("EIP").longValue()!=RET;budget++){if(budget>8)throw new Exception("unexpectedBUILDcalls");long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();if(pc==callback){long actual=read(e,sp+4,4);if(actual!=record)throw new Exception("wrong BUILD slot");calls++;observed=(short)read(e,record+54,2);}else if(pc!=0x45ccb0&&pc!=0x40a790&&pc!=0x40a9c0)throw new Exception("unexpected "+Long.toHexString(pc));long ret=read(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());}
   out.println(scenario+","+slot+","+(short)value+","+calls+","+observed+","+(short)read(e,record+54,2));
  }
 }finally{e.dispose();}}}
}
