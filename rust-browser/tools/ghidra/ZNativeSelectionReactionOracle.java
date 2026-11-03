// Real43a830/43a7a0 +43a360/3f0/480 and11b90. Geometry queries,45ef00
// and icon feedback are recorded boundaries. No dispatch table is stubbed.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeSelectionReactionOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,V=U+192,R=0x60ec00L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<1024;s++){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  e.writeMemory(toAddr(U),new byte[384]);e.writeMemory(toAddr(R),new byte[240*72]);for(int i=0;i<240;i++){for(int j=0;j<21;j++)put(e,R+i*72+19+j,1,255);put(e,R+i*72+19,1,s%11==0?255:(i*7)%240);}put(e,R+19,1,0);
  int fam=new int[]{4,6,24,25,30,32,34,36,38}[s%9],cls=new int[]{0,15,16,17,18}[s/9%5],saved=new int[]{0,15,16,17,18}[s/45%5],opcode=new int[]{1,2,5,4}[s/225%4];int x=s%2==0?3456:-1-s%100,y=s%240,z=300+s%30,code=new int[]{0,51,59}[s/7%3];
  put(e,U+0x30,1,fam);put(e,U+0x7c,1,cls);put(e,U+0x8e,1,saved);put(e,U+0x99,1,s*13);put(e,U+0x28,4,0x98765432);put(e,V+0x30,1,s%7==0?38:4);put(e,V+0xa8,2,new int[]{0xacb,0xac3,65535,111}[s%4]);put(e,V+0x28,4,0x12345678);put(e,0x4d1204,4,V);put(e,0x62dc04,4,U);put(e,0x56ed47,1,opcode);put(e,0x62dc00,4,x);put(e,0x62dc3c,4,y);put(e,0x62dc48,2,z);put(e,0x58aaaa+(short)(((z&0xfff0)*8)+(y>>4)),1,code);
  for(long p:new long[]{RET,0x4093e0,0x41c290,0x45ef00,0x438740})e.setBreakpoint(toAddr(p));e.writeRegister("ESP",S);put(e,S,4,RET);if(!e.run(toAddr(0x43a830),null,monitor))throw new Exception(e.getLastError());int query=0,events=0,ex=0,ey=0,ez=0,eo=0,icon=-1,stops=0;
  while(e.readRegister("EIP").longValue()!=RET){if(++stops>10)throw new Exception("too many stops");long pc=e.readRegister("EIP").longValue();if(pc==0x4093e0){query++;e.writeRegister("EAX",(s&2)!=0?1:0);}else if(pc==0x41c290){long a=e.readRegister("ECX").longValue(),b=e.readRegister("EDX").longValue();put(e,a,2,read(e,a,2)+3);put(e,b,2,read(e,b,2)-5);e.writeRegister("EAX",(s&4)!=0?1:0);}else if(pc==0x45ef00){events++;eo=(byte)read(e,0x56ed47,1);ex=(short)read(e,0x62dc00,2);ey=(short)read(e,0x62dc3c,2);ez=(short)read(e,0x62dc48,2);}else{icon=e.readRegister("EDX").intValue()&65535;}resume(e);}
  out.println("{\"scenario\":"+s+",\"handled\":"+(e.readRegister("EAX").intValue()&255)+",\"query\":"+query+",\"events\":"+events+",\"opcode\":"+eo+",\"x\":"+ex+",\"y\":"+ey+",\"z\":"+ez+",\"icon\":"+icon+",\"outerX\":"+(short)read(e,0x62dc00,2)+",\"outerY\":"+(short)read(e,0x62dc3c,2)+",\"outerZ\":"+(short)read(e,0x62dc48,2)+",\"context\":"+read(e,U+0x28,4)+"}");
 }finally{e.dispose();}}}}
}
