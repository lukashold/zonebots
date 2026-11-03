// Real176b0/174e0/13340.126e0 physical validation,60910 RNG and17610
// final unit creation are explicit oracle boundaries; their inputs are recorded.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeFactoryPlacementOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x5e30c8L,B=0x5c3f0cL,P=0x58aaaaL;
 static final int[][]SOCKET={{17,170},{113,1},{113,170},{17,1},{113,49},{17,49}},MARKER={{10,20,45},{11,20,45},{12,21,53},{13,21,53},{14,22,54},{15,22,54},{16,23,55},{17,23,55}};
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void resume(EmulatorHelper e)throws Exception{long s=e.readRegister("ESP").longValue(),r=read(e,s,4);e.writeRegister("ESP",s+4);if(!e.run(toAddr(r),null,monitor))throw new Exception(e.getLastError());}
 long hash(byte[]b){int h=0x811c9dc5;for(byte n:b)h=(h^(n&255))*16777619;return h&0xffffffffL;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<128;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(U),new byte[240*192]);e.writeMemory(toAddr(B),new byte[30]);e.writeMemory(toAddr(P),new byte[16384]);for(int[]m:MARKER)put(e,P+m[1]*128+m[0],1,m[2]|128);
   put(e,B,2,10);put(e,B+2,2,20);put(e,B+4,2,18);put(e,B+6,2,26);put(e,B+24,1,s&1);put(e,0x622d54,2,1000);put(e,0x622d56,2,1000);put(e,0x622c7a,2,s%20);put(e,0x622dc4,1,0);put(e,0x4a897c,2,new int[]{-1,0,63,127,140}[(s>>4)%5]);
   if((s&2)!=0)for(int i=0;i<6;i++){put(e,U+i*192+0x31,1,5);put(e,U+i*192+0x7c,1,2);put(e,U+i*192+0x2c,2,1000+SOCKET[i][0]);put(e,U+i*192+0x2e,2,1000+SOCKET[i][1]);}
   for(long a:new long[]{RET,0x4126e0,0x460910,0x417610})e.setBreakpoint(toAddr(a));e.writeRegister("ESP",S);put(e,S,4,RET);e.writeRegister("ECX",s%19);e.writeRegister("EDX",B);if(!e.run(toAddr(0x4176b0),null,monitor))throw new Exception(e.getLastError());
   int random=0,checks=0,placed=0,x=0,y=0,steps=0;
   while(e.readRegister("EIP").longValue()!=RET){if(++steps>40)throw new Exception("too many stops");long pc=e.readRegister("EIP").longValue();
    if(pc==0x4126e0){checks++;long px=e.readRegister("ECX").longValue(),py=e.readRegister("EDX").longValue();int cx=(short)read(e,px,2),cy=(short)read(e,py,2);boolean invalid=(s&4)!=0||(((cx>>4)+(cy>>4)+s)%3==0);if(!invalid&&(s&8)!=0){put(e,px,2,cx+16);put(e,py,2,cy-16);}e.writeRegister("EAX",invalid?1:0);}
    else if(pc==0x460910){random++;e.writeRegister("EAX",(random*29+s*17)%128);}
    else if(pc==0x417610){placed++;x=(short)e.readRegister("ECX").intValue();y=(short)e.readRegister("EDX").intValue();e.writeRegister("EAX",0);}
    else throw new Exception("Unexpected "+Long.toHexString(pc));resume(e);
   }
   out.println("{\"scenario\":"+s+",\"placed\":"+placed+",\"x\":"+x+",\"y\":"+y+",\"checks\":"+checks+",\"random\":"+random+",\"failed\":"+(e.readRegister("EAX").intValue()&255)+",\"terrain\":"+hash(e.readMemory(toAddr(P),16384))+"}");
  }finally{e.dispose();}
 }}}
}
