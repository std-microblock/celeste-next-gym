import { useEffect, useRef, useState } from 'react'
import type { KeyBindings, GymMap, SimState } from '../model'
import type { VisualTheme } from '../visualThemes'
import { createBlankGymMap, createTrainingProject, openTrainingWorkspace, saveTrainingWorkspace, type TrainingProject } from '../training/editorProject'
import { MapEditor } from './MapEditor'
import { TrainingFlowEditor } from './TrainingFlowEditor'

function downloadJson(name: string, value: unknown): void {
  const url = URL.createObjectURL(new Blob([`${JSON.stringify(value, null, 2)}\n`], { type: 'application/json' }))
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = name
  anchor.click()
  URL.revokeObjectURL(url)
}

export interface EditorWorkspaceProps {
  map: GymMap
  state: SimState
  frame: number
  theme: VisualTheme
  bindings: KeyBindings
  experiencing: boolean
  ready: boolean
  onMapChange: (map: GymMap) => void
  onExperienceChange: (experiencing: boolean) => void
  onResetExperience: () => void
}

export function EditorWorkspace(props: EditorWorkspaceProps) {
  const [section, setSection] = useState<'map' | 'training'>('map')
  const [projects, setProjects] = useState<TrainingProject[]>(() => [createTrainingProject(createBlankGymMap())])
  const [projectIndex, setProjectIndex] = useState(0)
  const [directory, setDirectory] = useState<FileSystemDirectoryHandle | null>(null)
  const [saveState, setSaveState] = useState<'memory' | 'dirty' | 'saving' | 'saved' | 'error'>('memory')
  const [notice, setNotice] = useState('当前为浏览器内存项目')
  const importRef = useRef<HTMLInputElement>(null)
  const revision = useRef(0)
  const current = projects[projectIndex] ?? projects[0]

  useEffect(() => {
    props.onMapChange(projects[0].map)
    // The editor deliberately starts from a small blank room instead of the playground.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const replaceProjects = (next: TrainingProject[], nextIndex = projectIndex) => {
    revision.current += 1
    setProjects(next)
    setProjectIndex(Math.min(Math.max(0, nextIndex), Math.max(0, next.length - 1)))
    if (directory) setSaveState('dirty')
  }

  const changeCurrent = (project: TrainingProject) => {
    const next = projects.map((candidate, index) => index === projectIndex ? project : candidate)
    replaceProjects(next)
    props.onMapChange(project.map)
  }

  useEffect(() => {
    if (!directory || saveState !== 'dirty') return
    const savingRevision = revision.current
    const timer = window.setTimeout(() => {
      setSaveState('saving')
      void saveTrainingWorkspace(directory, projects).then(() => {
        if (revision.current === savingRevision) {
          setSaveState('saved')
          setNotice(`已自动保存到 ${directory.name}`)
        } else {
          setSaveState('dirty')
        }
      }).catch((error: Error) => {
        setSaveState('error')
        setNotice(`自动保存失败：${error.message}`)
      })
    }, 500)
    return () => window.clearTimeout(timer)
  }, [directory, projects, saveState])

  const openFolder = async () => {
    if (!window.showDirectoryPicker) {
      setNotice('当前浏览器不支持 File System Access；可继续用 JSON 导入/导出')
      return
    }
    try {
      const handle = await window.showDirectoryPicker({ id: 'celeste-gym-training-workspace', mode: 'readwrite' })
      const loaded = await openTrainingWorkspace(handle, props.map)
      setDirectory(handle)
      revision.current += 1
      setProjects(loaded)
      setProjectIndex(0)
      props.onMapChange(loaded[0].map)
      setSaveState('saved')
      setNotice(`已打开 ${handle.name} · ${loaded.length} 个训练项目 · 自动保存开启`)
    } catch (error) {
      if (error instanceof DOMException && error.name === 'AbortError') return
      setSaveState('error')
      setNotice(error instanceof Error ? error.message : '文件夹打开失败')
    }
  }

  const saveNow = async () => {
    if (!directory) { setNotice('请先打开一个文件夹以启用自动保存'); return }
    try {
      const savingRevision = revision.current
      setSaveState('saving')
      await saveTrainingWorkspace(directory, projects)
      if (revision.current === savingRevision) {
        setSaveState('saved')
        setNotice(`已保存到 ${directory.name}`)
      } else {
        setSaveState('dirty')
      }
    } catch (error) {
      setSaveState('error')
      setNotice(error instanceof Error ? error.message : '保存失败')
    }
  }

  const importFiles = async (files: FileList) => {
    try {
      let map: GymMap | undefined
      let training: TrainingProject['training'] | undefined
      for (const file of Array.from(files)) {
        const value = JSON.parse(await file.text()) as Partial<GymMap> & Partial<TrainingProject['training']>
        if (Array.isArray(value.solids) && Array.isArray(value.entities) && value.bounds && value.spawn) map = value as GymMap
        if (value.version === 2 && Array.isArray(value.modules) && value.finish) training = value as TrainingProject['training']
      }
      const project = createTrainingProject(map ?? current.map, training ?? current.training)
      changeCurrent(project)
      setNotice(`已导入 ${files.length} 个 JSON 文件`)
    } catch (error) {
      setNotice(error instanceof Error ? `导入失败：${error.message}` : '导入失败')
    } finally {
      if (importRef.current) importRef.current.value = ''
    }
  }

  const addProject = () => {
    const project = createTrainingProject(createBlankGymMap(`untitled-room-${projects.length + 1}`))
    project.id = `training-map-${projects.length + 1}`
    project.training.id = project.id
    project.mapFileName = `${project.id}.map.json`
    project.trainingFileName = `${project.id}.training.json`
    replaceProjects([...projects, project], projects.length)
    props.onMapChange(project.map)
    setNotice(`已创建 ${project.id}`)
  }

  if (!current) return null
  return <div className="editor-workspace-shell">
    <nav className="editor-workspace-nav" aria-label="编辑器工作区">
      <div className="editor-section-tabs"><button className={section === 'map' ? 'active' : ''} onClick={() => setSection('map')}>地图</button><button className={section === 'training' ? 'active' : ''} onClick={() => setSection('training')}>训练流程</button></div>
      <select aria-label="训练项目" value={projectIndex} onChange={(event) => {
        const index = Number(event.target.value)
        setProjectIndex(index)
        props.onMapChange(projects[index].map)
      }}>{projects.map((project, index) => <option value={index} key={`${project.id}-${index}`}>{project.training.title}</option>)}</select>
      <button onClick={addProject}>新建</button>
      <button onClick={() => void openFolder()}>打开文件夹</button>
      <button onClick={() => importRef.current?.click()}>导入 JSON</button>
      <input ref={importRef} hidden multiple type="file" accept="application/json,.json" onChange={(event) => event.target.files && void importFiles(event.target.files)} />
      <button onClick={() => { downloadJson(current.mapFileName, current.map); downloadJson(current.trainingFileName, current.training); setNotice('地图与训练脚本已导出') }}>导出</button>
      <button disabled={!directory || saveState === 'saving'} onClick={() => void saveNow()}>保存</button>
      <span className={`editor-save-state ${saveState}`}><i />{saveState === 'saving' ? '保存中' : saveState === 'saved' ? '已自动保存' : saveState === 'dirty' ? '等待保存' : saveState === 'error' ? '保存错误' : '内存项目'}</span>
      <output title={notice}>{notice}</output>
    </nav>
    {section === 'map' ? <MapEditor
      map={current.map}
      state={props.state}
      frame={props.frame}
      theme={props.theme}
      experiencing={props.experiencing}
      ready={props.ready}
      onChange={(map) => changeCurrent({ ...current, map })}
      onExperienceChange={props.onExperienceChange}
      onResetExperience={props.onResetExperience}
    /> : <TrainingFlowEditor
      project={current}
      theme={props.theme}
      bindings={props.bindings}
      ready={props.ready}
      onChange={changeCurrent}
    />}
  </div>
}
