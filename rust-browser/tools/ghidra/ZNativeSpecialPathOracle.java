// Native1b950 role-dependent Tough and saved-driver APC path masks.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeSpecialPathOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void call(EmulatorHelper e,long f)throws Exception{put(e,S,4,RET);e.writeRegister("ESP",S);e.writeRegister("ECX",U);e.writeRegister("EIP",f);e.getEmulator().setHalt(false);for(int n=0;n<20000;n++){if(e.readRegister("EIP").longValue()==RET)return;if(!e.step(monitor))throw new Exception(e.getLastError());}throw new Exception("CPU budget");}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int world=0;world<2;world++)for(int team=0;team<3;team++)for(int kind=0;kind<2;kind++)for(int role:new int[]{-1,0,1,3}){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[0xc0]);for(int code:new int[]{9,38,39,42})put(e,0x499888+code,1,world==1?255:0);call(e,0x45f6a0);put(e,U+0x7c,1,kind==0?3:1);put(e,U+0x30,1,kind==0?4:32);put(e,U+0x5e,1,role);put(e,U+0x8e,1,role);put(e,U+0x31,1,team);call(e,0x41b950);long table=read(e,U+0x18,4);StringBuilder line=new StringBuilder(world+","+team+","+kind+","+role);for(int code=0;code<67;code++)line.append(',').append((byte)read(e,table+code,1));out.println(line);out.flush();}finally{e.dispose();}
  }
 }}
}
