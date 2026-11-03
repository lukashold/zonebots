// Execute actual59c00→59af0 and its full21-member invalidation loop.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeGroupDirectionOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,R=0x60ec00L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 int hash(int h,int n){return(h^n)*16777619;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int s=0;s<128;s++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    byte[]r=new byte[72];for(int j=0;j<72;j++)r[j]=(byte)(s*7+j);for(int role=0;role<21;role++)r[19+role]=(byte)((role+s)%5==0?255:role);e.writeMemory(toAddr(R),r);put(e,R+10,2,s%8*64);
    e.writeMemory(toAddr(U),new byte[240*192]);for(int i=0;i<21;i++)put(e,U+i*192+0x31,1,(s*11+i*7)&255);put(e,0x632194,4,R);
    int dx=(s*13)%27-13,dy=(s*7)%27-13;e.writeRegister("ECX",500);e.writeRegister("EDX",800);e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,500+dx);put(e,S+8,4,800+dy);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(0x459c00),null,monitor))throw new Exception(e.getLastError());
    int rh=0x811c9dc5,mh=0x811c9dc5;for(int j=0;j<72;j++)rh=hash(rh,(int)read(e,R+j,1));for(int i=0;i<21;i++)mh=hash(mh,(int)read(e,U+i*192+0x31,1));
    out.println("{\"scenario\":"+s+",\"roster\":"+Integer.toUnsignedString(rh)+",\"members\":"+Integer.toUnsignedString(mh)+"}");
   }finally{e.dispose();}
  }
 }}
}
