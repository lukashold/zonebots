// Complete41f8f0/41f610. Only DLLrand,420200 and41dbf0 are recorded leaves.
import ghidra.app.script.GhidraScript;import ghidra.app.emulator.EmulatorHelper;import java.io.*;
public class ZNativeBuildingFragmentsOracle extends GhidraScript{
 static final long S=0x7fff0000L,RET=0x12345000L,CR=RET+0x1000,P=0x1800000L,T=P+0x1000,Q=P+0x3000;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void set(byte[]b,int at,int n,int v){for(int i=0;i<n;i++)b[at+i]=(byte)(v>>(i*8));}
 long mix(long h,long v){return((h^(v&0xffffffffL))*16777619L)&0xffffffffL;}
 long hash(byte[]b){long h=2166136261L;for(byte v:b)h=mix(h,v&255);return h;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  put(e,0x4740ac,4,CR);for(long a:new long[]{RET,CR,0x420200,0x41dbf0})e.setBreakpoint(toAddr(a));
  for(int mode=0;mode<2;mode++)for(int s=0;s<1024;s++){
   byte[]pool=new byte[900];for(int i=0;i<50;i++)for(int j=0;j<18;j++)pool[i*18+j]=(byte)(s*13+i*17+j*29);
   if(mode==0){int active=new int[]{0,1,17,49,50}[s%5];for(int i=0;i<50;i++)set(pool,i*18,2,i<active?100+i:-1);}
   else{int count=new int[]{1,2,17,49,50}[s%5];for(int i=0;i<50;i++){int at=i*18;set(pool,at,2,i==count?-1:new int[]{-33,-32,-31,0,400,800,32767,-32768}[(s+i)%8]);set(pool,at+2,2,new int[]{-33,-32,-31,0,300,600,32767,-32768}[(s/3+i)%8]);set(pool,at+8,2,new int[]{1782,1778,100,32767,-32768}[(s+i)%5]);set(pool,at+10,2,new int[]{-32768,-2,-1,0,1,100,32767}[(s/5+i)%7]);pool[at+12]=(byte)new int[]{-128,-1,0,1,4,127}[(s/7+i)%6];pool[at+13]=(byte)new int[]{0,1,3,255}[(s/11+i)%4];pool[at+14]=(byte)((s+i*3)&7);pool[at+15]=(byte)new int[]{0,1,2,127,255}[(s/13+i)%5];pool[at+16]=(byte)new int[]{0,1,3,255}[(s/17+i)%4];}}
   e.writeMemory(toAddr(P),pool);byte[]templates=new byte[13*12];for(int i=0;i<13;i++){set(templates,i*12,2,i==12?-1:i*7-15);set(templates,i*12+2,2,i*11-35);set(templates,i*12+4,2,i%2==0?1782:-1782);set(templates,i*12+6,2,new int[]{-32768,-2,-1,0,1,100,32767}[(s+i)%7]);set(templates,i*12+8,2,i+s);set(templates,i*12+10,2,i*3+s);}e.writeMemory(toAddr(T),templates);
   int crt=0;for(int tick=0;tick<(mode==0?1:4);tick++){
    e.writeRegister("ESP",S);put(e,S,4,RET);put(e,0x62b3a4,4,Q);put(e,0x622cb8,2,s%3==0?500:0);put(e,0x622cba,2,s%3==0?300:0);put(e,0x498c24,2,800);put(e,0x498c20,2,600);put(e,0x622dc4,1,s%2);put(e,0x4a6e72,1,13);long event=2166136261L;int calls=0;
    if(mode==0){e.writeRegister("ECX",(short)(s*71));e.writeRegister("EDX",(short)(s*137));put(e,S+4,4,T);put(e,S+8,4,P);put(e,S+12,4,new int[]{-32768,-1,0,1,300,32767}[s%6]);}
    else{e.writeRegister("ECX",P);e.writeRegister("EDX",new int[]{-32768,-2,-1,0,1,10,100,32767}[(s/19+tick)%8]);}
    if(!e.run(toAddr(mode==0?0x41f8f0:0x41f610),null,monitor))throw new Exception(e.getLastError());
    while(e.readRegister("EIP").longValue()!=RET){long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue();if(pc==CR){e.writeRegister("EAX",(s*3571+crt++*131+0x1743)&32767);}else{calls++;event=mix(event,pc);if(pc==0x420200){event=mix(event,(short)e.readRegister("EDX").intValue());event=mix(event,(short)read(e,sp+4,2));event=mix(event,read(e,0x4a6e72,1));event=mix(event,e.readRegister("ECX").longValue());event=mix(event,read(e,sp+8,4));}else if(pc==0x41dbf0){event=mix(event,(short)e.readRegister("ECX").intValue());event=mix(event,(short)e.readRegister("EDX").intValue());}else throw new Exception("Unexpected "+Long.toHexString(pc));}long ret=read(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());}
    int draws=(int)((read(e,0x62b3a4,4)-Q)/8);out.println("{\"mode\":"+mode+",\"scenario\":"+s+",\"tick\":"+tick+",\"pool\":"+hash(e.readMemory(toAddr(P),900))+",\"draws\":"+draws+",\"queue\":"+(draws==0?2166136261L:hash(e.readMemory(toAddr(Q),draws*8)))+",\"crt\":"+crt+",\"calls\":"+calls+",\"events\":"+event+",\"flag\":"+read(e,0x4a6e72,1)+"}");
   }
  }
 }finally{e.dispose();}}}
}
