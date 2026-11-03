// Complete41ba00 numeric projection against all19 original weapon records.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
import java.nio.file.*;
public class ZNativeHardwareProfileOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,H=0x59e6c2L;
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 long hash(byte[]b){int h=0x811c9dc5;for(byte n:b)h=(h^(n&255))*16777619;return h&0xffffffffL;}
 public void run()throws Exception{byte[]robots=Files.readAllBytes(Paths.get("C:/GOG Games/Z/robots.dat"));int[]profiles={0,1,2,3,4,5,7,8,9,6,11,13,16,10,12,14,15,17,18};try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<304;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(0x615498),robots);byte[]b=new byte[44];for(int i=0;i<44;i++)b[i]=(byte)(s*7+i*13);e.writeMemory(toAddr(H),b);e.writeMemory(toAddr(U),new byte[192]);put(e,0x632378,4,0x61aa88+profiles[s%19]*42);e.setBreakpoint(toAddr(RET));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",H);e.writeRegister("EDX",U);if(!e.run(toAddr(0x41ba00),null,monitor))throw new Exception(e.getLastError());out.println("{\"scenario\":"+s+",\"hash\":"+hash(e.readMemory(toAddr(H),44))+"}");
  }finally{e.dispose();}
 }}}
}
