// Native43ad60/43ac70/43ab30/43aaf0 and43a600. Selection mutations execute
// without stubs. Dispersal supplies explicit41c290/43a520 query outcomes.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeSelectionOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,R=0x60ec00L,P=0x5a1edeL,M=0x4d1208L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void call(EmulatorHelper e,long pc,long cx,long dx,long...args)throws Exception{e.writeRegister("ESP",S);put(e,S,4,RET);for(int i=0;i<args.length;i++)put(e,S+4+4*i,4,args[i]);e.writeRegister("ECX",cx);e.writeRegister("EDX",dx);if(!e.run(toAddr(pc),null,monitor))throw new Exception(e.getLastError());}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 long checksum(EmulatorHelper e)throws Exception{long h=2166136261L;for(byte b:e.readMemory(toAddr(P),240*241))h=((h^(b&255))*16777619L)&0xffffffffL;return h;}
 void reset(EmulatorHelper e)throws Exception{byte[]b=new byte[240*241];java.util.Arrays.fill(b,(byte)255);e.writeMemory(toAddr(P),b);e.writeMemory(toAddr(U),new byte[192]);e.writeMemory(toAddr(R),new byte[240*72]);for(int i=0;i<240;i++){for(int j=0;j<21;j++)put(e,R+i*72+0x19+j,1,255);}put(e,0x632194,4,R);put(e,0x5849fa,1,255);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int s=0;s<256;s++){EmulatorHelper e=new EmulatorHelper(currentProgram);try{reset(e);e.setBreakpoint(toAddr(RET));
   for(int row=0;row<4;row++){put(e,P+row*241,1,(s+row*67)%240);for(int j=0;j<5;j++)put(e,P+row*241+1+j*7,1,(s+row*31+j*13)%240);}
   int g=(s*17)%256;put(e,S+0x100,4,0);put(e,S+0x104,2,0xffff);call(e,0x43ad60,g,S+0x100,S+0x104);int success=(e.readRegister("EAX").intValue()&255)==0?1:0;int row=success==0?-1:(int)((read(e,S+0x100,4)-P)/241),member=(short)read(e,S+0x104,2);
   int rem=(s+13)%240;put(e,S+0x104,1,0xee);call(e,0x43ac70,rem,P+(s%4)*241,S+0x104);int remaining=(int)read(e,S+0x104,1);long removed=checksum(e);
   put(e,U+0x5e,1,255);put(e,U+0x5f,1,(s+31)%240);call(e,0x43aaf0,U,0);long promoted=checksum(e);
   int add=(s*23)%240;put(e,U+0x5f,1,add);put(e,U+0x7c,1,s%9==0?2:0);put(e,U+0x30,1,s%11==0?36:4);put(e,0x5849fa,1,s%3==0?255:5);call(e,0x43ab30,U,0);int added=(e.readRegister("EAX").intValue()&255)==0?1:0;
   out.println("{\"kind\":\"rows\",\"scenario\":"+s+",\"found\":"+success+",\"row\":"+row+",\"member\":"+member+",\"remaining\":"+remaining+",\"removed\":"+removed+",\"promoted\":"+promoted+",\"added\":"+added+",\"insertion\":"+read(e,0x5849fa,1)+",\"final\":"+checksum(e)+"}");
  }finally{e.dispose();}}
  for(int s=0;s<128;s++){EmulatorHelper e=new EmulatorHelper(currentProgram);try{reset(e);put(e,P,1,3);put(e,P+1,1,3);put(e,P+2,1,7);put(e,P+3,1,9);put(e,U+0x5f,1,s%7==0?3:7);put(e,U+0x7c,1,s%3==0?1:0);int x=s%13==0?-20:200+s%5,y=300+s%7;put(e,0x62dc00,4,x);put(e,0x62dc3c,4,y);put(e,0x622d54,2,s%11==0?x+100:1000);put(e,0x622d56,2,s%11==0?y+100:1000);put(e,0x56ed47,1,s%2==0?1:5);for(int i=0;i<240;i++)put(e,M+i,1,s%17==0?1:(i<s%25?1:0));
   for(long pc:new long[]{RET,0x41c290,0x43a520})e.setBreakpoint(toAddr(pc));call(e,0x43a600,U,0);int b=0,p=0,stops=0;while(e.readRegister("EIP").longValue()!=RET){if(++stops>2500)throw new Exception("too many stops");long pc=e.readRegister("EIP").longValue();if(pc==0x41c290){b++;boolean hit=s%19==0||b<=s%3;put(e,0x5c5e38+4,1,0);put(e,read(e,e.readRegister("ESP").longValue()+4,4),4,0x5c5e38);if(!hit){long a=e.readRegister("ECX").longValue(),d=e.readRegister("EDX").longValue();put(e,a,2,read(e,a,2)+s%5);put(e,d,2,read(e,d,2)-s%4);}e.writeRegister("EAX",hit?1:0);}else{p++;e.writeRegister("EAX",s%23==0||p<=s%4?1:0);}resume(e);}
   long mask=2166136261L;for(byte v:e.readMemory(toAddr(M),240))mask=((mask^(v&255))*16777619L)&0xffffffffL;
   out.println("{\"kind\":\"disperse\",\"scenario\":"+s+",\"x\":"+(short)read(e,0x62dc00,2)+",\"y\":"+(short)read(e,0x62dc3c,2)+",\"building\":"+b+",\"physical\":"+p+",\"mask\":"+mask+"}");
  }finally{e.dispose();}}
 }}
}
