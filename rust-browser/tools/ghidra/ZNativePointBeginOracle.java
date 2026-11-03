//53b80 point initialization with real411b90 inventory and52a40 phase gates.
//59af0 heading is the sole isolated pure-math dependency (constant5).
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativePointBeginOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,G=0x60ec00L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void call(EmulatorHelper e)throws Exception{put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",U);e.writeRegister("EIP",0x453b80);e.getEmulator().setHalt(false);for(int n=0;n<50000;n++){long pc=e.readRegister("EIP").longValue();if(pc==RET)return;if(pc==0x459af0){long sp=e.readRegister("ESP").longValue();e.writeRegister("EAX",5);e.writeRegister("EIP",read(e,sp,4));e.writeRegister("ESP",sp+4);continue;}if(!e.step(monitor))throw new Exception(e.getLastError());}throw new Exception("CPU budget at "+e.readRegister("EIP").toString(16));}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int cls:new int[]{0,3,15,16})for(int role:new int[]{-1,0})for(int flags:new int[]{0,8})for(int scenario=0;scenario<32;scenario++){
 EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[192*6]);e.writeMemory(toAddr(G),new byte[72]);for(int j=0;j<21;j++)put(e,G+19+j,1,255);int[]phases={1,2,20,21,73,74,23,25,46,66};for(int j=0;j<6;j++){long p=U+j*192;put(e,G+19+j,1,(j+scenario)%7==0?255:j);put(e,p+0x31,1,(j+scenario)%7==1?0:4);put(e,p+0x35,1,phases[(j+scenario)%10]);put(e,p+0x32,2,j-2);put(e,p+0x3c,1,7);put(e,p+0x39,1,2);put(e,p+0x6a,2,11);put(e,p+0x6c,2,22);put(e,p+0x99,1,(j+scenario)%5==0?1:0);put(e,p+0x5f,1,0);}
 put(e,U+0x7c,1,cls);put(e,U+0x5e,1,role);put(e,U+0x5c,1,12);put(e,U+0x9c,1,flags);put(e,U+0x50,4,300);put(e,U+0x54,2,400);put(e,0x632194,4,G);put(e,0x4d02b2,1,0);put(e,0x4d02b3,1,0);call(e);StringBuilder row=new StringBuilder(cls+","+role+","+flags+","+scenario+","+(byte)read(e,U+0x5c,1)+","+read(e,U+0x9c,1));for(int j=0;j<6;j++){long p=U+j*192;row.append(","+read(e,p+0x35,1)+","+(short)read(e,p+0x32,2)+","+(byte)read(e,p+0x3c,1)+","+read(e,p+0x39,1)+","+(short)read(e,p+0x6a,2)+","+(short)read(e,p+0x6c,2));}out.println(row);out.flush();}finally{e.dispose();}
 }}}
}
