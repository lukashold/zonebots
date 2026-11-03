// Actual45f070 cancel/select commands, including real40a8a0/41a310 costs.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
import java.nio.file.*;
public class ZNativeFactoryCreditOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,F=0x5c5e38L,B=0x5c3f0cL;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void call(EmulatorHelper e,int product)throws Exception{put(e,0x62dc00,2,product);e.writeRegister("ESP",S);put(e,S,4,RET);if(!e.run(toAddr(0x45f070),null,monitor))throw new Exception(e.getLastError());}
 void output(PrintWriter out,EmulatorHelper e,int s,String kind)throws Exception{out.println("{\"scenario\":"+s+",\"kind\":\""+kind+"\",\"product\":"+(byte)read(e,F+7,1)+",\"saved\":"+(byte)read(e,F+68,1)+",\"remaining\":"+(int)read(e,F+36,4)+",\"total\":"+(int)read(e,F+40,4)+",\"credit\":"+(int)read(e,F+44,4)+",\"animation\":"+(short)read(e,F+10,2)+"}");}
 public void run()throws Exception{
  byte[]robots=Files.readAllBytes(Paths.get("C:/GOG Games/Z/robots.dat")),weights=Files.readAllBytes(Paths.get("C:/GOG Games/Z/mult.dat"));
  try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<128;s++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(0x615498),robots);e.writeMemory(toAddr(0x4a8980),weights);e.writeMemory(toAddr(F),new byte[88]);e.writeMemory(toAddr(B),new byte[30]);int owner=s%2,current=s%19;
    put(e,F+4,1,s%3==0?1:4);put(e,F+7,1,current);put(e,F+32,4,B);put(e,F+36,4,100+s*7);put(e,F+40,4,1000+s*13);put(e,F+10,2,17);put(e,B+8,1,owner);put(e,0x622dba,2,12);put(e,0x622dc4,1,(s&2)!=0?1:0);put(e,0x62dc4c,4,0);put(e,0x62dc3c,2,owner);put(e,0x56ed47,1,255);e.setBreakpoint(toAddr(RET));
    call(e,-1);output(out,e,s,"cancel");call(e,(s&4)!=0?current:(s+3)%19);output(out,e,s,"select");
   }finally{e.dispose();}
  }}
 }
}
