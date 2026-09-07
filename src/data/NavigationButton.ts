import {House, BookOpen, Plus, LayoutDashboard, Settings2 } from 'lucide-react'
import { NavigationButtonType } from '../types/NavigationButton'

export const buttons: NavigationButtonType[] = [
    {icon: House, label: 'Home', to: '/'},
    {icon: BookOpen, label: 'Review', to: '/review'},
    {icon: Plus, label: 'Create', to: '/create'},
    {icon: LayoutDashboard, label: 'Dashboard', to: '/dashboard'},
    {icon: Settings2, label: 'Settings', to: '/settings'},
]