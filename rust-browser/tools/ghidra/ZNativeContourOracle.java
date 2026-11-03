// Executes native infantry/vehicle local contour callbacks with synthetic
// permission and target-ray masks. No native path function is replaced.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeContourOracle extends GhidraScript{
 static final long RET=0x12345000L,S=0x7fff0000L,U=0x1000000L,T=U+0x1000,P=U+0x2000,M=U+0x4000,C=0x4d15f0L;
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int vehicle=0;vehicle<2;vehicle++)for(int mode=0;mode<3;mode++)for(int direction=0;direction<4;direction++){
   EmulatorHelper e=new EmulatorHelper(currentProgram);try{
    e.writeMemory(toAddr(U),new byte[0x9000]);e.writeMemory(toAddr(C),new byte[102]);byte[]map=new byte[16384],mask=new byte[16384];for(int y=0;y<128;y++)for(int x=0;x<128;x++){boolean blocked=x>=30&&x<=34&&y>=20&&y<=40;if(mode==1)blocked|=x>=30&&x<=45&&y>=40&&y<=41;if(mode==2)blocked|=x>=35&&x<=38&&y>=15&&y<=25;map[y*128+x]=(byte)(blocked?1:0);mask[y*128+x]=(byte)((direction<2?y>=29&&y<=31&&x>=35&&x<=70:x>=31&&x<=33&&y>=42&&y<=70)?255:65);}e.writeMemory(toAddr(0x58aaaa),map);e.writeMemory(toAddr(M),mask);put(e,T+1,1,255);put(e,U+0x18,4,T);put(e,0x63288c,4,U);put(e,0x633948,4,M);put(e,0x4d18c4,4,C);put(e,0x4d15e8,4,C+2);
    int cell=direction<2?(vehicle==1?28:29)+30*128:32+(vehicle==1?18:19)*128;int wall=direction<2?30+30*128:32+20*128;int step=new int[]{-128,128,1,-1}[direction];put(e,P+0x14,4,cell);put(e,P+0x18,4,wall);put(e,P+0x10,4,step);put(e,P+4,4,256);put(e,P+0x1c,4,M+cell);put(e,M+cell,1,0);e.setBreakpoint(toAddr(RET));long[]cores={0x43d130,0x43d2b0,0x43d430,0x43d5b0},wrappers={0x43d7c0,0x43d790,0x43d760,0x43d730};int edge=direction;
    for(int tick=1;tick<=20;tick++){
     e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,0);put(e,0x633a18,4,75);e.writeRegister("ECX",vehicle==1?P:U);e.writeRegister("EDX",vehicle==1?T:P);if(!e.run(toAddr(vehicle==1?cores[edge]:0x43c8d0),null,monitor))throw new Exception(e.getLastError());int status=e.readRegister("EAX").intValue()&255;if(vehicle==1){long w=read(e,0x4d133c,4);for(int i=0;i<4;i++)if(w==wrappers[i])edge=i;}int count=(byte)read(e,C+1,1),num=Math.abs(count);StringBuilder points=new StringBuilder();for(int i=0;i<num;i++){if(i>0)points.append(',');points.append((short)read(e,C+2+i*2,2));}long hash=2166136261L;byte[]visited=e.readMemory(toAddr(M),16384);for(byte v:visited)hash=((hash^(v&255L))*16777619L)&0xffffffffL;
     out.println("{\"vehicle\":"+vehicle+",\"mode\":"+mode+",\"direction\":"+direction+",\"tick\":"+tick+",\"status\":"+status+",\"cell\":"+(int)read(e,P+0x14,4)+",\"wall\":"+(int)read(e,P+0x18,4)+",\"step\":"+(int)read(e,P+0x10,4)+",\"steps\":"+read(e,P+4,4)+",\"edge\":"+edge+",\"score\":"+read(e,C,1)+",\"count\":"+count+",\"points\":["+points+"],\"maskHash\":"+hash+"}");
     if(status==0)break;
    }
   }finally{e.dispose();}
  }
 }}
}
