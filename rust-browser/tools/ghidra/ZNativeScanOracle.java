// Executes the original signed directional distance scanner0044ce00.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
import java.math.BigInteger;
public class ZNativeScanOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{
  try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
   for(int direction=0;direction<8;direction++)for(int dx:new int[]{-257,-129,-65,-17,-8,-1,0,1,7,8,17,65,129,257})for(int dy:new int[]{-129,-17,-1,0,1,17,129}){
    EmulatorHelper e=new EmulatorHelper(currentProgram);
    try{
     e.writeMemory(toAddr(U),new byte[0x180]);put(e,U+0x2c,2,500);put(e,U+0x2e,2,500);put(e,U+0x69,1,4);
     put(e,U+0xc0+0x2c,2,500+dx);put(e,U+0xc0+0x2e,2,500+dy);put(e,U+0xc0+0x31,1,5);
     put(e,0x62d3a8,1,1);put(e,0x62d3a9,1,255);put(e,0x635b70,4,read(e,0x49a1ec+direction*4,4));
     for(long a:new long[]{RET,0x473264,0x432320})e.setBreakpoint(toAddr(a));
     e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",U);
     if(!e.run(toAddr(0x44ce00),null,monitor))throw new Exception(e.getLastError());
     while(e.readRegister("EIP").longValue()!=RET){long a=e.readRegister("EIP").longValue();if(a==0x432320)e.writeRegister("EAX",0);else if(a==0x473264){long bits=e.readRegister("XMM0").longValue();double value=Math.sqrt(Double.longBitsToDouble(bits));e.writeRegister("XMM0",BigInteger.valueOf(Double.doubleToRawLongBits(value)));}else throw new Exception("Unexpected scan stop");resume(e);}
     long result=e.readRegister("EAX").longValue()&65535;
     out.println("{\"direction\":"+direction+",\"dx\":"+dx+",\"dy\":"+dy+",\"slot\":"+(result>>8)+",\"distance\":"+(result&255)+"}");
    }finally{e.dispose();}
   }
  }
 }
}
