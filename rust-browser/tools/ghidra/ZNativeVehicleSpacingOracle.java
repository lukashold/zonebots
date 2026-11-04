// Full461d10/45a9f0/461b30/461bb0/41c290. Only60910 RNG and525a0
// line installation are explicit boundaries; each actual call is recorded.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
import java.util.Arrays;
public class ZNativeVehicleSpacingOracle extends GhidraScript{
 static final long RET=0x12345000L,SP=0x7fff0000L,U=0x5e30c8L,W=0x71000000L,G=0x60ec00L,B=0x5c5e38L;
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 long get(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 int mix(int h,int n){return(h^n)*16777619;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  for(long a:new long[]{RET,0x460910,0x4525a0})e.setBreakpoint(toAddr(a));
  for(int s=0;s<1024;s++){
   e.writeMemory(toAddr(U),new byte[240*192]);e.writeMemory(toAddr(W),new byte[240*64]);e.writeMemory(toAddr(G),new byte[240*72]);e.writeMemory(toAddr(B),new byte[35*88]);
   byte[]plane=new byte[16384];Arrays.fill(plane,(byte)((s/8)%128|(s%2)*128));e.writeMemory(toAddr(0x58aaaa),plane);
   for(int k=0;k<35;k++)put(e,B+k*88+4,1,-1);
   put(e,B,2,480);put(e,B+2,2,480);put(e,B+4,1,s%3==0?0:1);put(e,B+30,2,s%5==0?0:1000);put(e,0x4a69f7,1,1);
   int n=s%32==0?240:8;int[][]d={{0,0},{0,0},{7,0},{8,0},{0,7},{32,0},{64,0},{-32,0}};
   for(int k=0;k<n;k++){long p=U+k*192;int x=n==240?100+(k%20)*64:500+d[k][0],y=n==240?100+(k/20)*64:500+d[k][1];int team=k<2?s%4:(s+k)%4,flags=team|4;if(n!=240&&(s+k)%13==0)flags&=~4;
    put(e,p+44,2,x);put(e,p+46,2,y);put(e,p+49,1,flags);put(e,p+0x7c,1,n!=240&&(s+k)%17==0?0:1);put(e,p+0x71,1,n!=240&&(s+k)%19==0?2:0);put(e,p+0x5f,1,k);put(e,G+k*72+58,1,(s+k)%7==0?1:0);put(e,p+8,4,W+k*64);put(e,W+k*64+23,1,(s+k)%11==0?1:0);
   }
   e.writeRegister("ESP",SP);put(e,SP,4,RET);int rng=0,calls=0,hash=0x811c9dc5;
   if(!e.run(toAddr(0x461d10),null,monitor))throw new Exception(e.getLastError());
   while(e.readRegister("EIP").longValue()!=RET){long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();int pop=0;
    if(pc==0x460910){e.writeRegister("EAX",(s*7919L)^(rng*2027L)^0x7731a5c9L);rng++;}
    else if(pc==0x4525a0){int slot=(int)((e.readRegister("ECX").longValue()-U)/192),x=(short)e.readRegister("EDX").intValue(),y=(short)get(e,sp+4,2);hash=mix(mix(mix(hash,slot),x),y);calls++;}
    else throw new Exception("Unexpected boundary "+Long.toHexString(pc));long next=get(e,sp,4);e.writeRegister("ESP",sp+4+pop);if(!e.run(toAddr(next),null,monitor))throw new Exception(e.getLastError());
   }
   int count=(byte)get(e,0x586976,1),cache=0x811c9dc5;for(int k=0;k<(count&255);k++){long p=0x4d1920L+k*20,unit=get(e,p,4);cache=mix(cache,(int)((unit-U)/192));cache=mix(cache,(int)get(e,p+4,4));cache=mix(cache,(int)get(e,p+8,4));cache=mix(cache,(int)get(e,unit+49,1)&3);}
   out.println(s+","+count+","+Integer.toUnsignedString(cache)+","+rng+","+calls+","+Integer.toUnsignedString(hash));
  }
 }finally{e.dispose();}}}
}
