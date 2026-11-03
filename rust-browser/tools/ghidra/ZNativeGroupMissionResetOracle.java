// Actual4b530 plus460430's exact strategy-row loop, with nonzero old memory.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeGroupMissionResetOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,R=0x60ec00L,G=0x5c47b8L;
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 long hash(byte[]b){int h=0x811c9dc5;for(byte n:b)h=(h^(n&255))*16777619;return h&0xffffffffL;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<16;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{byte[]r=new byte[240*72],g=new byte[240*24];for(int i=0;i<r.length;i++)r[i]=(byte)(s*11+i*7);for(int i=0;i<g.length;i++)g[i]=(byte)(s*19+i*3);e.writeMemory(toAddr(R),r);e.writeMemory(toAddr(G),g);e.setBreakpoint(toAddr(RET));e.writeRegister("ESP",S);put(e,S,4,RET);if(!e.run(toAddr(0x44b530),null,monitor))throw new Exception(e.getLastError());e.setBreakpoint(toAddr(0x460687));if(!e.run(toAddr(0x46066e),null,monitor))throw new Exception(e.getLastError());
   out.println("{\"scenario\":"+s+",\"roster\":"+hash(e.readMemory(toAddr(R),r.length))+",\"strategy\":"+hash(e.readMemory(toAddr(G),g.length))+"}");
  }finally{e.dispose();}
 }}}
}
