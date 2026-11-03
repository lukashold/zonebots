// Permission tables selected by native initialization and direction wrappers.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeCollisionMaskOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void call(EmulatorHelper e,long f)throws Exception{put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",U);e.writeRegister("EIP",f);e.getEmulator().setHalt(false);for(int n=0;n<20000;n++){if(e.readRegister("EIP").longValue()==RET)return;if(!e.step(monitor))throw new Exception(e.getLastError());}throw new Exception("CPU budget at "+e.readRegister("EIP").toString(16));}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int kind=0;kind<3;kind++)for(int team=0;team<3;team++)for(int direction=0;direction<8;direction++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[0xc0]);call(e,0x45f6a0);put(e,U+0x7c,1,kind==0?0:1);put(e,U+0x30,1,kind==2?36:34);put(e,U+0x31,1,team);put(e,U+0x39,1,direction);call(e,0x41b950);call(e,0x45a2e0);long table=read(e,U+0x1c,4);for(int d:new int[]{0,45,53,54,55})put(e,table+team+d,1,0);StringBuilder line=new StringBuilder(kind+","+team+","+direction);for(int code=0;code<67;code++)line.append(',').append((byte)read(e,table+code,1));out.println(line);out.flush();}finally{e.dispose();}
  }
 }}
}
