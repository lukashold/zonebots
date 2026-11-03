// Numeric native road-region lookup0043b5f0 and authored next-hop chain0043b6f0.
// Synthetic records only; no original map records are copied into the trace.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeRoadGraphOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,H=U+0x1000,N=0x5eee28L,G=0x5b6da4L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 void invoke(EmulatorHelper e,long a)throws Exception{e.writeRegister("ESP",S);put(e,S,4,RET);e.setBreakpoint(toAddr(RET));if(!e.run(toAddr(a),null,monitor))throw new Exception(e.getLastError());}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int x:new int[]{0,99,100,200,300,400,401})for(int y:new int[]{0,99,100,200,300,400,401}){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.writeMemory(toAddr(G),new byte[96*136]);for(int i=0;i<3;i++){long r=G+i*136;put(e,r,2,i*100);put(e,r+2,2,i*100);put(e,r+4,2,i*100+200);put(e,r+6,2,i*100+200);}e.writeRegister("ECX",x);e.writeRegister("EDX",y);put(e,S+4,4,U);invoke(e,0x43b5f0);out.println("{\"kind\":\"region\",\"x\":"+x+",\"y\":"+y+",\"failed\":"+(e.readRegister("EAX").intValue()&255)+",\"region\":"+read(e,U,1)+"}");}finally{e.dispose();}
  }
  for(int variant=0;variant<3;variant++)for(int current=0;current<7;current++)for(int destination=0;destination<7;destination++){
   if(variant==2&&(current!=1||destination!=6))continue;EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(U),new byte[0x2000]);e.writeMemory(toAddr(G),new byte[96*136]);e.writeMemory(toAddr(N),new byte[100*8]);e.writeMemory(toAddr(0x4d1340),new byte[400]);put(e,0x633a24,4,0x4d1340);put(e,0x586867,1,0);put(e,0x4d0e33,1,0);put(e,H,4,N);for(int i=1;i<100;i++)put(e,N+i*8+4,4,-1);
    for(int i=0;i<96;i++){long r=G+i*136;for(int j=0;j<96;j++)put(e,r+40+j,1,255);if(i<7){put(e,r+8,2,(i+5)+(i+7)*128);for(int j=0;j<7;j++){int next=i+(j>i?1:-1);if(i==j)next=j;if(variant==1&&i==3)next=255;if(variant==2)next=i==1?2:1;put(e,r+40+j,1,next);}}}
    e.writeRegister("ECX",H);e.writeRegister("EDX",destination);put(e,S+4,4,current);invoke(e,0x43b6f0);StringBuilder points=new StringBuilder();int count=0;long p=read(e,N+4,4);while(p!=0&&p!=-1&&count<99){if(count++>0)points.append(',');points.append('[').append((short)read(e,p,2)).append(',').append((short)read(e,p+2,2)).append(']');p=read(e,p+4,4);}
    out.println("{\"kind\":\"chain\",\"variant\":"+variant+",\"current\":"+current+",\"destination\":"+destination+",\"failed\":"+(e.readRegister("EAX").intValue()&255)+",\"count\":"+count+",\"points\":["+points+"]}");
   }finally{e.dispose();}
  }
 }}
}
